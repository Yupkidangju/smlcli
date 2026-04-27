// [v0.1.0-beta.7] Phase 3 리팩토링 2차: mod.rs에서 도구 런타임 분리.
// LLM 응답에서 JSON 도구 호출을 파싱하고, 권한 검사(PermissionEngine)를 수행한 뒤,
// 허용된 도구는 자동 실행, 승인 필요 도구는 Approval 카드로 라우팅하는 전체 파이프라인을 캡슐화.
// 또한 사용자의 y/n 승인 입력 처리도 이 모듈에서 담당.

use super::{App, action, event_loop};

impl App {
    /// [v3.4.0] 파일시스템을 변경하는 도구 목록. 직렬화 큐(write_tool_queue)에 사용.
    /// [v3.4.0] Phase 44 완료: DeleteFile이 GLOBAL_REGISTRY에 정식 등록됨.
    /// GitCheckpoint는 스냅샷 보존 도구이므로 write 목록에서 의도적으로 제외.
    pub(crate) fn is_write_tool(name: &str) -> bool {
        matches!(
            name,
            "WriteFile" | "ReplaceFileContent" | "DeleteFile" | "ExecShell"
        )
    }

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
            let mut valid_tools = Vec::new();
            for (idx, call) in tool_calls.iter().enumerate() {
                let name = call.function.name.clone();
                // [v1.5.0] 잘못된 JSON 인자(Malformed Tool Call) 시 복구 피드백 루프 전송
                let args = match serde_json::from_str::<serde_json::Value>(&call.function.arguments)
                {
                    Ok(parsed) => parsed,
                    Err(e) => {
                        let err_msg = format!("Invalid JSON format in tool arguments: {}", e);
                        self.state
                            .runtime
                            .logs_buffer
                            .push(format!("[Tool Parse Error] {}", err_msg));
                        let _ = self
                            .action_tx
                            .try_send(crate::app::event_loop::Event::Action(
                                crate::app::action::Action::ToolError(
                                    crate::domain::error::ToolError::ExecutionFailure(err_msg),
                                    Some(call.id.clone()),
                                    idx,
                                ),
                            ));
                        continue;
                    }
                };
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

                valid_tools.push((tool_call, call.id.clone(), idx));
            }

            self.state.runtime.pending_tool_executions += valid_tools.len();
            for (tool_call, id, idx) in valid_tools {
                self.dispatch_tool_call(tool_call, Some(id), idx);
            }
        }
    }

    /// 파싱된 ToolCall에 대해 권한 검사를 수행하고, 결과에 따라 실행/승인대기/거부를 처리.
    /// [v2.5.0] permission 확인 후 적절한 타임라인 블록을 생성하는 구조로 통일.
    /// push-then-pop 패턴 제거 → Allow=ToolRun, Ask=Approval, Deny=ToolRun(Error).
    pub(crate) fn dispatch_tool_call(
        &mut self,
        tool_call: crate::domain::tool_result::ToolCall,
        tool_call_id: Option<String>,
        tool_index: usize,
    ) {
        let settings = self.state.domain.settings.clone().unwrap_or_default();
        let tool_name = Self::format_tool_name(&tool_call);
        let perm = crate::domain::permissions::PermissionEngine::check(&tool_call, &settings);

        match perm {
            crate::domain::permissions::PermissionResult::Allow => {
                // Allow: ToolRun 블록 생성 후 실행
                let mut block = crate::app::state::TimelineBlock::new(
                    crate::app::state::TimelineBlockKind::ToolRun,
                    tool_name.clone(),
                )
                .with_depth(1)
                .with_tool_call_id(tool_call_id.clone());
                block.status = crate::app::state::BlockStatus::Running;
                self.state.ui.timeline.push(block);

                if Self::is_write_tool(&tool_call.name) {
                    if self.state.runtime.is_write_tool_running {
                        self.state.runtime.write_tool_queue.push_back((
                            tool_call,
                            tool_call_id,
                            tool_index,
                        ));
                    } else {
                        self.state.runtime.is_write_tool_running = true;
                        self.execute_tool_async(tool_call, tool_call_id, tool_index);
                    }
                } else {
                    self.execute_tool_async(tool_call, tool_call_id, tool_index);
                }
            }
            crate::domain::permissions::PermissionResult::Ask => {
                if self.state.runtime.approval.pending_tool.is_none() {
                    // Ask (첫 번째): Approval 블록 직접 생성
                    let mut approval_block = crate::app::state::TimelineBlock::new(
                        crate::app::state::TimelineBlockKind::Approval,
                        tool_name.clone(),
                    )
                    .with_depth(1)
                    .with_tool_call_id(tool_call_id.clone());
                    approval_block.status = crate::app::state::BlockStatus::NeedsApproval;
                    approval_block
                        .body
                        .push(crate::app::state::BlockSection::Markdown(
                            Self::format_tool_detail(&tool_call),
                        ));
                    self.state.ui.timeline.push(approval_block);

                    self.state.runtime.approval.pending_tool = Some(tool_call.clone());
                    self.state.runtime.approval.pending_tool_call_id = tool_call_id;
                    self.state.runtime.approval.pending_tool_index = Some(tool_index);
                    self.state.runtime.approval.pending_since_ms = Some(super::App::unix_time_ms());
                    self.state.ui.show_inspector = true;

                    // Diff Preview 자동 매핑
                    if let Some(tool) =
                        crate::tools::registry::GLOBAL_REGISTRY.get_tool(&tool_call.name)
                    {
                        self.state.runtime.approval.diff_preview =
                            tool.generate_diff_preview(&tool_call.args);
                    }
                } else {
                    // Ask (대기열): 큐에 추가
                    self.state.runtime.approval.queued_approvals.push_back((
                        tool_call,
                        tool_call_id,
                        tool_index,
                    ));
                }
            }
            crate::domain::permissions::PermissionResult::Deny(reason) => {
                // Deny: 에러 상태의 ToolRun 블록 생성 후 결과 전송
                let mut block = crate::app::state::TimelineBlock::new(
                    crate::app::state::TimelineBlockKind::ToolRun,
                    tool_name.clone(),
                )
                .with_depth(1)
                .with_tool_call_id(tool_call_id.clone());
                block.status = crate::app::state::BlockStatus::Error;
                self.state.ui.timeline.push(block);

                let res = crate::domain::tool_result::ToolResult {
                    tool_name: tool_call.name.clone(),
                    stdout: String::new(),
                    stderr: format!("[Security Block] {}", reason),
                    exit_code: 1,
                    is_error: true,
                    tool_call_id: tool_call_id.clone(),
                    is_truncated: false,
                    original_size_bytes: None,
                    affected_paths: vec![],
                };
                let tx = self.action_tx.clone();
                tokio::spawn(async move {
                    let _ = tx
                        .send(event_loop::Event::Action(action::Action::ToolFinished(
                            Box::new(res),
                            tool_index,
                        )))
                        .await;
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
            // [v3.4.0] Phase 44: DeleteFile 타임라인 표시 포맷 추가
            "DeleteFile" => format!(
                "DeleteFile: {}",
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
                    .get("query")
                    .or_else(|| tool_call.args.get("pattern"))
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
        tool_index: usize,
    ) {
        // [v0.1.0-beta.18] Phase 9-A: ToolStarted — 타임라인 카드를 Running으로 갱신
        for block in self.state.ui.timeline.iter_mut().rev() {
            if block.kind == crate::app::state::TimelineBlockKind::ToolRun
                && block.tool_call_id == tool_call_id
                && block.status == crate::app::state::BlockStatus::Idle
            {
                block.status = crate::app::state::BlockStatus::Running;
                break;
            }
        }

        let tx = self.action_tx.clone();
        let token = crate::domain::permissions::PermissionToken::grant();

        // [v2.5.0] 병렬 도구별 독립 CancellationToken 생성 및 등록
        let cancel_token = tokio_util::sync::CancellationToken::new();
        let token_key = tool_call_id
            .clone()
            .unwrap_or_else(|| format!("tool_{}", tool_index));
        self.state
            .runtime
            .active_tool_cancel_tokens
            .insert(token_key, cancel_token.clone());

        // [v1.9.0] Phase 27: 터미널 타이틀 & 작업표시줄 진행률 동기화 (OSC)
        {
            use std::io::Write;
            let title_str = format!("\x1b]0;[smlcli] Executing: {}\x07", tool_call.name);
            let progress_str = "\x1b]9;4;1;100\x07";
            print!("{}{}", title_str, progress_str);
            let _ = std::io::stdout().flush();
        }

        // [v3.7.0] Phase 47 Task Q-3: AskClarification 도구 인터셉트.
        // 이 도구는 비동기 실행 대신 TUI 모달로 전환하여 사용자 답변을 수집.
        // ShowQuestionnaire Action을 통해 이벤트 루프에서 처리됨.
        if tool_call.name == "AskClarification" {
            match serde_json::from_value::<crate::domain::questionnaire::AskClarificationArgs>(
                tool_call.args.clone(),
            ) {
                Ok(clarification) => {
                    let tx = self.action_tx.clone();
                    let questions = clarification.questions;
                    let tcid = tool_call_id.clone();
                    tokio::spawn(async move {
                        let _ = tx
                            .send(event_loop::Event::Action(
                                action::Action::ShowQuestionnaire(questions, tcid, tool_index),
                            ))
                            .await;
                    });
                    return;
                }
                Err(e) => {
                    // 인자 파싱 실패 시 에러 반환
                    let tx = self.action_tx.clone();
                    tokio::spawn(async move {
                        let _ = tx
                            .send(event_loop::Event::Action(action::Action::ToolError(
                                crate::domain::error::ToolError::InvalidArguments(format!(
                                    "AskClarification 인자 파싱 실패: {}",
                                    e
                                )),
                                tool_call_id,
                                tool_index,
                            )))
                            .await;
                    });
                    return;
                }
            }
        }

        // [v3.3.2] 감사 HIGH-3 수정: 역매핑 테이블 기반 MCP 라우팅.
        // 이전: mcp_clients 원본 서버명으로 prefix match → 정규화된 스키마 이름과 불일치.
        // 수정: mcp_tool_name_map에서 전체 도구명을 직접 조회하여
        //       (sanitized_server, original_tool_name)을 한 번에 확인.
        //       mcp_clients도 정규화 서버명을 key로 사용하므로 완전 일치 보장.
        let is_mcp = tool_call.name.starts_with("mcp_");
        let mcp_route = if is_mcp {
            self.state
                .runtime
                .mcp_tool_name_map
                .get(&tool_call.name)
                .and_then(|(sanitized_server, original_tool_name)| {
                    self.state
                        .runtime
                        .mcp_clients
                        .get(sanitized_server)
                        .map(|c| (c.clone(), original_tool_name.clone()))
                })
        } else {
            None
        };

        tokio::spawn(async move {
            if let Some((client, actual_tool_name)) = mcp_route {
                // [v3.3.2] 역매핑에서 복원한 원본 MCP 도구명으로 call_tool 호출.
                // 정규화된 이름이 아닌 MCP 서버가 인식하는 원래 이름 사용.
                match client
                    .call_tool(&actual_tool_name, tool_call.args.clone())
                    .await
                {
                    Ok(output) => {
                        let res = crate::domain::tool_result::ToolResult {
                            tool_name: tool_call.name.clone(),
                            tool_call_id: tool_call_id.clone(),
                            stdout: output.to_string(),
                            stderr: String::new(),
                            exit_code: 0,
                            is_error: false,
                            is_truncated: false,
                            original_size_bytes: None,
                            affected_paths: vec![],
                        };
                        let _ = tx
                            .send(event_loop::Event::Action(action::Action::ToolFinished(
                                Box::new(res),
                                tool_index,
                            )))
                            .await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(event_loop::Event::Action(action::Action::ToolError(
                                crate::domain::error::ToolError::ExecutionFailure(e.to_string()),
                                tool_call_id,
                                tool_index,
                            )))
                            .await;
                    }
                }
            } else {
                match crate::tools::executor::execute_tool(tool_call, &token, cancel_token).await {
                    Ok(mut res) => {
                        res.tool_call_id = tool_call_id;
                        let _ = tx
                            .send(event_loop::Event::Action(action::Action::ToolFinished(
                                Box::new(res),
                                tool_index,
                            )))
                            .await;
                    }
                    Err(e) => {
                        // [v0.1.0-beta.21] ToolError 구조화: String 대신 도메인 타입 사용
                        let _ = tx
                            .send(event_loop::Event::Action(action::Action::ToolError(
                                crate::domain::error::ToolError::ExecutionFailure(e.to_string()),
                                tool_call_id,
                                tool_index,
                            )))
                            .await;
                    }
                }
            }
        });
    }

    /// 사용자가 도구 승인 카드에서 'y'(승인) 또는 'n'(거부)을 입력했을 때 처리.
    pub(crate) fn handle_tool_approval(&mut self, approved: bool) {
        // [v2.5.0] 상태 경합/만료 직후 입력 등 경계 조건에서 패닉 방지.
        // pending_tool이 None이면 이미 처리됐거나 만료된 상태이므로 조기 반환.
        let Some(tool) = self.state.runtime.approval.pending_tool.take() else {
            return;
        };
        let tool_call_id = self.state.runtime.approval.pending_tool_call_id.take();
        let tool_index = self
            .state
            .runtime
            .approval
            .pending_tool_index
            .take()
            .unwrap_or(0);
        self.state.runtime.approval.diff_preview = None;
        self.state.runtime.approval.pending_since_ms = None;

        if approved {
            let tool_name = Self::format_tool_name(&tool);
            let mut block = crate::app::state::TimelineBlock::new(
                crate::app::state::TimelineBlockKind::ToolRun,
                tool_name,
            )
            .with_depth(1)
            .with_tool_call_id(tool_call_id.clone());
            block.status = crate::app::state::BlockStatus::Running;
            self.state.ui.timeline.push(block);

            if Self::is_write_tool(&tool.name) {
                if self.state.runtime.is_write_tool_running {
                    self.state.runtime.write_tool_queue.push_back((
                        tool,
                        tool_call_id.clone(),
                        tool_index,
                    ));
                } else {
                    self.state.runtime.is_write_tool_running = true;
                    self.execute_tool_async(tool, tool_call_id.clone(), tool_index);
                }
            } else {
                self.execute_tool_async(tool, tool_call_id.clone(), tool_index);
            }

            self.state
                .domain
                .session
                .add_message(crate::providers::types::ChatMessage {
                    role: crate::providers::types::Role::System,
                    content: Some("Tool is running in background...".to_string()),
                    tool_calls: None,
                    tool_call_id: tool_call_id.clone(),
                    pinned: false,
                });
        } else {
            // [v0.1.0-beta.18] 거부: 타임라인에 SystemNotice 추가
            self.state.ui.timeline.push(
                crate::app::state::TimelineBlock::new(
                    crate::app::state::TimelineBlockKind::Notice,
                    "사용자가 도구 실행을 거부했습니다.",
                )
                .with_depth(1)
                .with_tool_call_id(tool_call_id.clone()),
            );

            let res = crate::domain::tool_result::ToolResult {
                tool_name: tool.name.clone(),
                stdout: String::new(),
                stderr: "Tool execution rejected by user.".to_string(),
                exit_code: 1,
                is_error: true,
                tool_call_id: tool_call_id.clone(),
                is_truncated: false,
                original_size_bytes: None,
                affected_paths: vec![],
            };
            let tx = self.action_tx.clone();
            tokio::spawn(async move {
                let _ = tx
                    .send(event_loop::Event::Action(action::Action::ToolFinished(
                        Box::new(res),
                        tool_index,
                    )))
                    .await;
            });
        }

        // Pop next queued approval if any
        if let Some((next_tool, next_id, next_idx)) =
            self.state.runtime.approval.queued_approvals.pop_front()
        {
            self.state.runtime.approval.pending_tool = Some(next_tool.clone());
            self.state.runtime.approval.pending_tool_call_id = next_id.clone();
            self.state.runtime.approval.pending_tool_index = Some(next_idx);
            self.state.runtime.approval.pending_since_ms = Some(super::App::unix_time_ms());
            if let Some(registry_tool) =
                crate::tools::registry::GLOBAL_REGISTRY.get_tool(&next_tool.name)
            {
                self.state.runtime.approval.diff_preview =
                    registry_tool.generate_diff_preview(&next_tool.args);
            }
            // [v2.5.0] 기존 블록 변형 대신 새 Approval 블록을 명시적으로 생성.
            // 직전에 Notice/ToolRun이 추가된 경우 해당 블록이 의도치 않게 변형되는 것을 방지.
            let mut approval_block = crate::app::state::TimelineBlock::new(
                crate::app::state::TimelineBlockKind::Approval,
                Self::format_tool_name(&next_tool),
            );
            approval_block.status = crate::app::state::BlockStatus::NeedsApproval;
            approval_block.tool_call_id = next_id;
            approval_block
                .body
                .push(crate::app::state::BlockSection::Markdown(
                    Self::format_tool_detail(&next_tool),
                ));
            self.state.ui.timeline.push(approval_block);
            self.state.ui.show_inspector = true;
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
                self.state.runtime.pending_tool_executions += 1;
                if matches!(perm, crate::domain::permissions::PermissionResult::Allow) {
                    self.execute_tool_async(tool_call, None, 0);
                } else {
                    // [v2.5.0] Ask 경로에서 Approval 타임라인 카드 생성.
                    // Inspector 승인 UI뿐 아니라 Timeline에서도 승인 대기 맥락이 표시됨.
                    let mut approval_block = crate::app::state::TimelineBlock::new(
                        crate::app::state::TimelineBlockKind::Approval,
                        format!("! {}", cmd),
                    );
                    approval_block.status = crate::app::state::BlockStatus::NeedsApproval;
                    approval_block
                        .body
                        .push(crate::app::state::BlockSection::Markdown(format!(
                            "직접 셸 실행 승인 대기: `{}`",
                            cmd
                        )));
                    self.state.ui.timeline.push(approval_block);

                    self.state.runtime.approval.pending_tool = Some(tool_call);
                    self.state.runtime.approval.pending_tool_index = Some(0);
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
