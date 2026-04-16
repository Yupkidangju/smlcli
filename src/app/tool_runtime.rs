// [v0.1.0-beta.7] Phase 3 리팩토링 2차: mod.rs에서 도구 런타임 분리.
// LLM 응답에서 JSON 도구 호출을 파싱하고, 권한 검사(PermissionEngine)를 수행한 뒤,
// 허용된 도구는 자동 실행, 승인 필요 도구는 Approval 카드로 라우팅하는 전체 파이프라인을 캡슐화.
// 또한 사용자의 y/n 승인 입력 처리도 이 모듈에서 담당.

use super::{App, action, event_loop};

impl App {
    /// LLM 응답 내의 ```json``` 블록을 파싱하여 도구 호출을 감지하고,
    /// 권한 정책에 따라 자동 실행/승인 대기/거부를 결정하는 파이프라인.
    ///
    /// 흐름:
    /// 1. 응답 텍스트에서 ```json ... ``` 패턴을 탐색
    /// 2. serde_json으로 ToolCall로 역직렬화 시도
    /// 3. PermissionEngine.check()로 정책 판단
    ///    4-A. Allow → 즉시 비동기 실행, 결과를 ToolFinished 이벤트로 반환
    ///    4-B. Ask → approval.pending_tool에 등록, Inspector 강제 오픈 + Diff Preview 생성
    ///    4-C. Deny → 타임라인에 보안 차단 메시지 추가
    pub(crate) fn process_tool_calls_from_response(&mut self, content: &str) {
        // ```json 블록 탐색
        if let Some(start_idx) = content.find("```json") {
            let block = &content[start_idx + 7..];
            if let Some(end_idx) = block.find("```") {
                let json_str = block[..end_idx].trim();
                if let Ok(tool_call) =
                    serde_json::from_str::<crate::domain::tool_result::ToolCall>(json_str)
                {
                    self.dispatch_tool_call(tool_call);
                }
            }
        }
    }

    /// 파싱된 ToolCall에 대해 권한 검사를 수행하고, 결과에 따라 실행/승인대기/거부를 처리.
    pub(crate) fn dispatch_tool_call(&mut self, tool_call: crate::domain::tool_result::ToolCall) {
        let settings = self.state.settings.clone().unwrap_or_default();

        // [v0.1.0-beta.18] Phase 9-A: ToolQueued 타임라인 엔트리 추가
        let tool_name = format!("{:?}", &tool_call).chars().take(30).collect::<String>();
        self.state.timeline.push(
            crate::app::state::TimelineEntry::now(
                crate::app::state::TimelineEntryKind::ToolCard {
                    tool_name: tool_name.clone(),
                    status: crate::app::state::ToolStatus::Queued,
                    summary: "권한 검사 중...".to_string(),
                },
            ),
        );

        let perm = crate::domain::permissions::PermissionEngine::check(&tool_call, &settings);

        match perm {
            crate::domain::permissions::PermissionResult::Allow => {
                // 자동 실행: 비동기 spawn으로 TUI 프리징 방지
                self.execute_tool_async(tool_call);
            }
            crate::domain::permissions::PermissionResult::Ask => {
                // 승인 대기: Inspector 패널 강제 오픈 + Diff Preview 자동 생성
                // [v0.1.0-beta.18] 타임라인 ToolCard를 ApprovalCard로 대체
                if let Some(last) = self.state.timeline.last_mut() {
                    last.kind = crate::app::state::TimelineEntryKind::ApprovalCard {
                        tool_name: tool_name.clone(),
                        detail: "사용자 승인 대기 중 (y/n)".to_string(),
                    };
                }
                self.state.approval.pending_tool = Some(tool_call.clone());
                self.state.show_inspector = true;

                // Diff Preview 자동 매핑: 파일 수정 도구인 경우 미리보기 생성
                match tool_call {
                    crate::domain::tool_result::ToolCall::ReplaceFileContent {
                        path,
                        target_content,
                        replacement_content,
                    } => {
                        let old_text = std::fs::read_to_string(&path).unwrap_or_default();
                        let diff = crate::tools::file_ops::generate_diff(
                            &old_text,
                            &old_text.replace(&target_content, &replacement_content),
                        );
                        self.state.approval.diff_preview = Some(diff);
                    }
                    crate::domain::tool_result::ToolCall::WriteFile { path, content, .. } => {
                        let diff = crate::tools::file_ops::write_file_preview(&path, &content)
                            .unwrap_or_default();
                        self.state.approval.diff_preview = Some(diff);
                    }
                    _ => {}
                }
            }
            crate::domain::permissions::PermissionResult::Deny(reason) => {
                // [v0.1.0-beta.18] 타임라인 ToolCard를 Error 상태로 갱신
                for entry in self.state.timeline.iter_mut().rev() {
                    if let crate::app::state::TimelineEntryKind::ToolCard {
                        ref mut status,
                        ref mut summary,
                        ..
                    } = entry.kind
                        && *status == crate::app::state::ToolStatus::Queued
                    {
                        *status = crate::app::state::ToolStatus::Error;
                        *summary = format!("🛡 {}", reason);
                        break;
                    }
                }
                // 거부: 보안 차단 메시지를 세션에 표시
                self.state
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: format!("[Security Block] {}", reason),
                        pinned: false,
                    });
            }
        }
    }

    /// 도구를 비동기로 실행하고, 완료/에러 결과를 이벤트 루프에 전송.
    pub(crate) fn execute_tool_async(&mut self, tool_call: crate::domain::tool_result::ToolCall) {
        // [v0.1.0-beta.18] Phase 9-A: ToolStarted — 타임라인 카드를 Running으로 갱신
        let tool_name = format!("{:?}", &tool_call).chars().take(30).collect::<String>();
        for entry in self.state.timeline.iter_mut().rev() {
            if let crate::app::state::TimelineEntryKind::ToolCard {
                ref mut status, ..
            } = entry.kind
                && *status == crate::app::state::ToolStatus::Queued
            {
                *status = crate::app::state::ToolStatus::Running;
                break;
            }
        }

        let tx = self.action_tx.clone();
        let token = crate::domain::permissions::PermissionToken::grant();

        tokio::spawn(async move {
            match crate::tools::executor::execute_tool(tool_call, &token).await {
                Ok(res) => {
                    let _ = tx
                        .send(event_loop::Event::Action(action::Action::ToolFinished(res)))
                        .await;
                }
                Err(e) => {
                    let _ = tx
                        .send(event_loop::Event::Action(action::Action::ToolError(
                            e.to_string(),
                        )))
                        .await;
                }
            }
        });
    }

    /// 사용자가 도구 승인 카드에서 'y'(승인) 또는 'n'(거부)을 입력했을 때 처리.
    /// 승인 시 execute_tool_async를 호출하고, 거부 시 pending_tool을 해제.
    pub(crate) fn handle_tool_approval(&mut self, approved: bool) {
        if approved {
            let tool = self.state.approval.pending_tool.take().unwrap();
            self.state.approval.diff_preview = None;

            // [v0.1.0-beta.18] 승인 완료: ApprovalCard를 ToolCard(Running)로 전환
            let tool_name = format!("{:?}", &tool).chars().take(30).collect::<String>();
            self.state.timeline.push(
                crate::app::state::TimelineEntry::now(
                    crate::app::state::TimelineEntryKind::ToolCard {
                        tool_name,
                        status: crate::app::state::ToolStatus::Running,
                        summary: "실행 중...".to_string(),
                    },
                ),
            );

            self.execute_tool_async(tool);

            self.state
                .session
                .add_message(crate::providers::types::ChatMessage {
                    role: crate::providers::types::Role::System,
                    content: "Tool is running in background...".to_string(),
                    pinned: false,
                });
        } else {
            self.state.approval.pending_tool = None;
            self.state.approval.diff_preview = None;

            // [v0.1.0-beta.18] 거부: 타임라인에 SystemNotice 추가
            self.state.timeline.push(
                crate::app::state::TimelineEntry::now(
                    crate::app::state::TimelineEntryKind::SystemNotice(
                        "사용자가 도구 실행을 거부했습니다.".to_string(),
                    ),
                ),
            );

            self.state
                .session
                .add_message(crate::providers::types::ChatMessage {
                    role: crate::providers::types::Role::System,
                    content: "Tool execution rejected by user.".to_string(),
                    pinned: false,
                });
        }
    }

    /// Composer에서 '!' 접두사로 입력된 직접 셸 실행 요청을 처리.
    /// 권한 정책에 따라 즉시 실행, 승인 대기, 또는 거부를 결정.
    pub(crate) fn handle_direct_shell_execution(&mut self, cmd: String) {
        if cmd.is_empty() {
            return;
        }

        let settings = self.state.settings.clone().unwrap_or_default();
        let tool_call = crate::domain::tool_result::ToolCall::ExecShell {
            command: cmd.clone(),
            cwd: None,
            safe_to_auto_run: false,
        };
        let perm = crate::domain::permissions::PermissionEngine::check(&tool_call, &settings);

        match perm {
            crate::domain::permissions::PermissionResult::Allow
            | crate::domain::permissions::PermissionResult::Ask => {
                // 직접 셸 실행은 Allow가 아닐 경우 항상 Ask 처리됨
                if matches!(perm, crate::domain::permissions::PermissionResult::Allow) {
                    self.execute_tool_async(tool_call);
                } else {
                    self.state.approval.pending_tool = Some(tool_call);
                    self.state.show_inspector = true;
                }
            }
            crate::domain::permissions::PermissionResult::Deny(reason) => {
                self.state
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: format!("[Security Block] {}", reason),
                        pinned: false,
                    });
            }
        }
    }
}
