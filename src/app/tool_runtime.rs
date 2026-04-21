// [v0.1.0-beta.7] Phase 3 리팩토링 2차: mod.rs에서 도구 런타임 분리.
// LLM 응답에서 JSON 도구 호출을 파싱하고, 권한 검사(PermissionEngine)를 수행한 뒤,
// 허용된 도구는 자동 실행, 승인 필요 도구는 Approval 카드로 라우팅하는 전체 파이프라인을 캡슐화.
// 또한 사용자의 y/n 승인 입력 처리도 이 모듈에서 담당.

use super::{App, action, event_loop};

impl App {
    /// [v0.1.0-beta.22] 도구 호출 파싱 — 엄격한 후처리 계층.
    /// - fenced ```json 블록만 도구 호출로 인식한다 (bare JSON은 무시).
    /// - "tool" 필드가 존재하고 ToolCall serde 역직렬화에 성공해야 디스패치.
    /// - 빈 ExecShell 명령은 파싱 단계에서 사전 차단.
    /// - 자연어 설명 없이 JSON만 있는 응답은 경고를 로깅한다.
    pub(crate) fn process_tool_calls_from_response(
        &mut self,
        msg: &crate::providers::types::ChatMessage,
    ) {
        if !self.state.runtime.user_intent_actionable {
            self.state.runtime.logs_buffer.push(
                "[Harness] 비작업성 입력으로 분류되었지만, 모델이 구조화된 도구 호출을 반환하면 \
                 모델 판단을 우선한다."
                    .to_string(),
            );
        }

        if let Some(tool_calls) = &msg.tool_calls {
            let mut found_count = 0;
            for call in tool_calls {
                let name = call.function.name.clone();
                let args = serde_json::from_str::<serde_json::Value>(&call.function.arguments)
                    .unwrap_or_else(|_| serde_json::json!({}));
                let tool_call = crate::domain::tool_result::ToolCall { name, args };

                // [v0.1.0-beta.22] 빈 명령은 권한 검사 이전에 즉시 차단
                if tool_call.name == "ExecShell"
                    && let Some(cmd) = tool_call.args.get("command").and_then(|v| v.as_str())
                    && cmd.trim().is_empty()
                {
                    self.state
                        .runtime
                        .logs_buffer
                        .push("[Harness] ExecShell 빈 명령 감지 → 실행 차단됨.".to_string());
                    self.state.ui.timeline.push(
                        crate::app::state::TimelineBlock::new(
                            crate::app::state::TimelineBlockKind::Notice,
                            "⚠ 빈 명령은 실행할 수 없습니다.",
                        )
                        .with_depth(1),
                    );
                    continue;
                }

                found_count += 1;
                if found_count == 1 {
                    self.dispatch_tool_call(tool_call, Some(call.id.clone()));
                } else {
                    self.state.runtime.logs_buffer.push(format!(
                        "[Multi-Tool] 추가 도구 감지 (#{}) — 현재 단일 실행 모드",
                        found_count
                    ));
                }
            }
        }
    }

    /// 파싱된 ToolCall에 대해 권한 검사를 수행하고, 결과에 따라 실행/승인대기/거부를 처리.
    pub(crate) fn dispatch_tool_call(
        &mut self,
        tool_call: crate::domain::tool_result::ToolCall,
        tool_call_id: Option<String>,
    ) {
        let settings = self.state.domain.settings.clone().unwrap_or_default();

        let tool_name = Self::format_tool_name(&tool_call);
        let mut block = crate::app::state::TimelineBlock::new(
            crate::app::state::TimelineBlockKind::ToolRun,
            tool_name.clone(),
        )
        .with_depth(1);
        block.status = crate::app::state::BlockStatus::Running;
        self.state.ui.timeline.push(block);

        let perm = crate::domain::permissions::PermissionEngine::check(&tool_call, &settings);

        match perm {
            crate::domain::permissions::PermissionResult::Allow => {
                // 자동 실행
                self.execute_tool_async(tool_call, tool_call_id);
            }
            crate::domain::permissions::PermissionResult::Ask => {
                // 승인 대기
                if let Some(last) = self.state.ui.timeline.last_mut() {
                    last.kind = crate::app::state::TimelineBlockKind::Approval;
                    last.status = crate::app::state::BlockStatus::NeedsApproval;
                    last.body.push(crate::app::state::BlockSection::Markdown(
                        Self::format_tool_detail(&tool_call),
                    ));
                }
                self.state.runtime.approval.pending_tool = Some(tool_call.clone());
                self.state.runtime.approval.pending_tool_call_id = tool_call_id;
                self.state.runtime.approval.pending_since_ms = Some(super::App::unix_time_ms());
                self.state.ui.show_inspector = true;

                // Diff Preview 자동 매핑
                if let Some(tool) =
                    crate::tools::registry::GLOBAL_REGISTRY.get_tool(&tool_call.name)
                {
                    self.state.runtime.approval.diff_preview =
                        tool.generate_diff_preview(&tool_call.args);
                }
            }
            crate::domain::permissions::PermissionResult::Deny(reason) => {
                // 타임라인 ToolCard를 Error 상태로 갱신
                for block in self.state.ui.timeline.iter_mut().rev() {
                    if block.kind == crate::app::state::TimelineBlockKind::ToolRun
                        && block.status == crate::app::state::BlockStatus::Running
                    {
                        block.status = crate::app::state::BlockStatus::Error;
                        block
                            .body
                            .push(crate::app::state::BlockSection::Markdown(format!(
                                "🛡 {}",
                                reason
                            )));
                        break;
                    }
                }
                // 거부 메시지 추가
                self.state
                    .domain
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: Some(format!("[Security Block] {}", reason)),
                        tool_calls: None,
                        tool_call_id,
                        pinned: false,
                    });
            }
        }
    }

    /// [v0.1.0-beta.22] 도구 종류별 의미 있는 이름 생성 (전체 경로 포함, 최대 120자).
    /// 이전: Debug 포맷의 30자 절단 → 개선: 도구명 + 핵심 정보 전체 표시.
    pub(crate) fn format_tool_name(tool_call: &crate::domain::tool_result::ToolCall) -> String {
        let raw = match tool_call.name.as_str() {
            "ExecShell" => format!(
                "ExecShell: {}",
                tool_call
                    .args
                    .get("command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
            ),
            "ReadFile" => format!(
                "ReadFile: {}",
                tool_call
                    .args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
            ),
            "WriteFile" => format!(
                "WriteFile: {}",
                tool_call
                    .args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
            ),
            "ReplaceFileContent" => format!(
                "ReplaceFileContent: {}",
                tool_call
                    .args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
            ),
            "ListDir" => format!(
                "ListDir: {}",
                tool_call
                    .args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
            ),
            "GrepSearch" => format!(
                "GrepSearch: '{}' in {}",
                tool_call
                    .args
                    .get("pattern")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
                tool_call
                    .args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
            ),
            "Stat" => format!(
                "Stat: {}",
                tool_call
                    .args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
            ),
            "SysInfo" => "SysInfo".to_string(),
            _ => format!("{:?}", tool_call),
        };
        if raw.chars().count() > 120 {
            format!("{}...", raw.chars().take(117).collect::<String>())
        } else {
            raw
        }
    }

    /// [v0.1.0-beta.22] 승인 카드에 표시할 전체 도구 설명 (축약 없이).
    pub(crate) fn format_tool_detail(tool_call: &crate::domain::tool_result::ToolCall) -> String {
        if let Some(tool) = crate::tools::registry::GLOBAL_REGISTRY.get_tool(&tool_call.name) {
            tool.format_detail(&tool_call.args)
        } else {
            format!("승인 대기 (y/n) — 알 수 없는 도구: {}", tool_call.name)
        }
    }

    /// 도구를 비동기로 실행하고, 완료/에러 결과를 이벤트 루프에 전송.
    pub(crate) fn execute_tool_async(
        &mut self,
        tool_call: crate::domain::tool_result::ToolCall,
        tool_call_id: Option<String>,
    ) {
        // [v0.1.0-beta.18] Phase 9-A: ToolStarted — 타임라인 카드를 Running으로 갱신
        for block in self.state.ui.timeline.iter_mut().rev() {
            if block.kind == crate::app::state::TimelineBlockKind::ToolRun
                && block.status == crate::app::state::BlockStatus::Idle
            {
                block.status = crate::app::state::BlockStatus::Running;
                break;
            }
        }

        let tx = self.action_tx.clone();
        let token = crate::domain::permissions::PermissionToken::grant();

        // [v1.0.0] Graceful Cancellation 지원을 위한 토큰 생성
        let cancel_token = tokio_util::sync::CancellationToken::new();
        self.state.runtime.active_tool_cancel_token = Some(cancel_token.clone());

        tokio::spawn(async move {
            match crate::tools::executor::execute_tool(tool_call, &token, cancel_token).await {
                Ok(mut res) => {
                    res.tool_call_id = tool_call_id;
                    let _ = tx
                        .send(event_loop::Event::Action(action::Action::ToolFinished(
                            Box::new(res),
                        )))
                        .await;
                }
                Err(e) => {
                    // [v0.1.0-beta.21] ToolError 구조화: String 대신 도메인 타입 사용
                    let _ = tx
                        .send(event_loop::Event::Action(action::Action::ToolError(
                            crate::domain::error::ToolError::ExecutionFailure(e.to_string()),
                        )))
                        .await;
                }
            }
        });
    }

    /// 사용자가 도구 승인 카드에서 'y'(승인) 또는 'n'(거부)을 입력했을 때 처리.
    pub(crate) fn handle_tool_approval(&mut self, approved: bool) {
        if approved {
            let tool = self.state.runtime.approval.pending_tool.take().unwrap();
            let tool_call_id = self.state.runtime.approval.pending_tool_call_id.take();
            self.state.runtime.approval.diff_preview = None;
            self.state.runtime.approval.pending_since_ms = None;

            let tool_name = Self::format_tool_name(&tool);
            let mut block = crate::app::state::TimelineBlock::new(
                crate::app::state::TimelineBlockKind::ToolRun,
                tool_name,
            )
            .with_depth(1);
            block.status = crate::app::state::BlockStatus::Running;
            self.state.ui.timeline.push(block);

            self.execute_tool_async(tool, tool_call_id.clone());

            self.state
                .domain
                .session
                .add_message(crate::providers::types::ChatMessage {
                    role: crate::providers::types::Role::System,
                    content: Some("Tool is running in background...".to_string()),
                    tool_calls: None,
                    tool_call_id,
                    pinned: false,
                });
        } else {
            let tool_call_id = self.state.runtime.approval.pending_tool_call_id.take();
            self.state.runtime.approval.pending_tool = None;
            self.state.runtime.approval.diff_preview = None;
            self.state.runtime.approval.pending_since_ms = None;

            // [v0.1.0-beta.18] 거부: 타임라인에 SystemNotice 추가
            self.state.ui.timeline.push(
                crate::app::state::TimelineBlock::new(
                    crate::app::state::TimelineBlockKind::Notice,
                    "사용자가 도구 실행을 거부했습니다.",
                )
                .with_depth(1),
            );

            self.state
                .domain
                .session
                .add_message(crate::providers::types::ChatMessage {
                    role: crate::providers::types::Role::System,
                    content: Some("Tool execution rejected by user.".to_string()),
                    tool_calls: None,
                    tool_call_id,
                    pinned: false,
                });
        }
    }

    /// Composer에서 '!' 접두사로 입력된 직접 셸 실행 요청을 처리.
    pub(crate) fn handle_direct_shell_execution(&mut self, cmd: String) {
        if cmd.is_empty() {
            return;
        }

        let settings = self.state.domain.settings.clone().unwrap_or_default();
        let tool_call = crate::domain::tool_result::ToolCall {
            name: "ExecShell".to_string(),
            args: serde_json::json!({
                "command": cmd.clone(),
                "cwd": ".",
                "safe_to_auto_run": false
            }),
        };
        let perm = crate::domain::permissions::PermissionEngine::check(&tool_call, &settings);

        match perm {
            crate::domain::permissions::PermissionResult::Allow
            | crate::domain::permissions::PermissionResult::Ask => {
                if matches!(perm, crate::domain::permissions::PermissionResult::Allow) {
                    self.execute_tool_async(tool_call, None);
                } else {
                    self.state.runtime.approval.pending_tool = Some(tool_call);
                    self.state.runtime.approval.pending_since_ms = Some(super::App::unix_time_ms());
                    self.state.ui.show_inspector = true;
                }
            }
            crate::domain::permissions::PermissionResult::Deny(reason) => {
                self.state
                    .domain
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: Some(format!("[Security Block] {}", reason)),
                        tool_calls: None,
                        tool_call_id: None,
                        pinned: false,
                    });
            }
        }
    }
}
