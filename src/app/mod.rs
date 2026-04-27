// [v0.1.0-beta.7] Phase 3 리팩토링 완료: mod.rs를 이벤트 루프 오케스트레이터로 축소.
//
// 모듈 책임 분배:
// - mod.rs (이 파일): App 구조체 정의, 이벤트 루프(run), 액션 디스패치
// - command_router.rs: 슬래시 커맨드 파싱 및 실행 (12개 커맨드)
// - chat_runtime.rs: LLM 요청 조립, API 키 조회, Provider 디스패치
// - tool_runtime.rs: 도구 호출 파싱, 권한 검사, 비동기 실행, 승인 UI
// - wizard_controller.rs: Setup Wizard 상태 전이, Config 팝업 Enter 처리
//
// 이전 상태: 773줄의 God Object
// 현재 상태: ~250줄의 이벤트 루프 오케스트레이터 + 입력 핸들러

// [v3.4.0] Phase 44 Task D-2: TECH-DEBT 정리 완료. 모든 서브모듈 활성화됨.
pub mod action;
pub mod chat_runtime;
pub mod command_router;
pub mod event_loop;
pub mod state;
pub mod tool_runtime;
pub mod wizard_controller;

use crate::tui::layout::draw;
use anyhow::Result;
use event_loop::{Event, EventLoop};
use state::AppState;

pub struct App {
    pub state: AppState,
    pub action_tx: tokio::sync::mpsc::Sender<event_loop::Event>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MousePaneTarget {
    Timeline,
    Inspector,
    Composer,
    Other,
}

impl App {
    const MAX_AUTO_VERIFY_RETRIES: usize = 3;
    const AUTO_VERIFY_MODEL_DETAIL_CHARS: usize = 1200;
    const AUTO_VERIFY_NOTICE_CHARS: usize = 240;
    const APPROVAL_TTL_MS: u64 = 5 * 60 * 1000;

    /// OpenAI function tool name 최대 길이 (64자).
    /// https://platform.openai.com/docs/api-reference/chat/create
    pub(crate) const MAX_TOOL_NAME_LEN: usize = 64;

    /// [v3.3.1] 감사 MEDIUM-2: MCP 서버명/도구명을 OpenAI tool name 규격에 맞게 정규화.
    /// OpenAI API는 ^[a-zA-Z0-9_-]+$ 만 허용하므로, 비허용 문자를 '_'로 치환.
    /// 빈 문자열이면 "unnamed"으로 대체.
    /// [v3.3.3] pub(crate)로 변경: command_router.rs의 /mcp add 정규화 충돌 검사에서도 사용.
    /// [v3.3.5] 감사 HIGH-1: 길이 제한 없이 정규화만 수행. 길이 제한은 full_name 조립 시
    /// truncate_tool_full_name()에서 적용.
    pub(crate) fn sanitize_tool_name_part(raw: &str) -> String {
        let sanitized: String = raw
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        if sanitized.is_empty() {
            "unnamed".to_string()
        } else {
            sanitized
        }
    }

    /// [v3.3.5] 감사 HIGH-1: MCP full_name을 OpenAI 64자 제한에 맞게 truncate.
    /// full_name = "mcp_{sanitized_server}_{sanitized_tool}" 형식.
    /// 접두사 "mcp_"(4자) + "_"(1자) = 5자 고정.
    /// 충돌 접미사("_NN") 최대 4자를 예비하여, 실제 파트에 55자를 할당.
    /// server 파트를 27자, tool 파트를 나머지(최대 27자)로 제한.
    pub(crate) fn build_mcp_full_name(sanitized_server: &str, sanitized_tool: &str) -> String {
        // 접두사 "mcp_" + "_" = 5자, 접미사 예비 4자 → 파트 할당 55자
        const PREFIX_LEN: usize = 5; // "mcp_" + "_"
        const SUFFIX_RESERVE: usize = 4; // "_99" 등
        let max_parts = Self::MAX_TOOL_NAME_LEN - PREFIX_LEN - SUFFIX_RESERVE;
        let half = max_parts / 2;

        // 서버 파트: 최대 half 자
        let srv = if sanitized_server.len() > half {
            &sanitized_server[..half]
        } else {
            sanitized_server
        };
        // 도구 파트: 나머지 할당량
        let tool_max = max_parts - srv.len();
        let tool = if sanitized_tool.len() > tool_max {
            &sanitized_tool[..tool_max]
        } else {
            sanitized_tool
        };
        format!("mcp_{}_{}", srv, tool)
    }

    pub async fn new(tx: tokio::sync::mpsc::Sender<event_loop::Event>) -> Self {
        Self::normalize_workspace_dir();
        let mut app = Self {
            state: AppState::new_async().await,
            action_tx: tx.clone(),
        };
        app.check_trust_gate();
        app.refresh_repo_map_if_needed(true);

        // [v2.4.0] Phase 32: 백그라운드 무소음 자가 진단(Silent Health Check)
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let report = crate::infra::doctor::DoctorReport::run_diagnostics().await;
            if report.has_issues() {
                let _ = tx_clone
                    .send(crate::app::event_loop::Event::Action(
                        crate::app::action::Action::SilentHealthCheckFailed,
                    ))
                    .await;
            }
        });
        // [v3.3.0] Phase 43: MCP 클라이언트 비동기 로드
        if let Some(settings) = &app.state.domain.settings {
            // [v3.3.4] 감사 MEDIUM-2 수정: 앱 시작 시 config.toml에서 로드된 MCP 서버들의
            // 정규화명 충돌 검증. config.toml에 'foo.bar'와 'foo_bar'가 동시에 있으면
            // 런타임에서 mcp_clients가 overwrite되어 도구가 잘못된 서버로 라우팅될 수 있음.
            //
            // [v3.3.5] 감사 MEDIUM-2 수정: 동일 서버명 중복 시 첫 번째는 로드, 후순위만 skip.
            // 이전: skipped_servers에 원본명 저장 → 동일명 두 번이면 둘 다 skip됨.
            // 수정: index 기반 skip set으로 교체하여 "최소 하나는 로드" 보장.
            // [v3.3.5] 감사 LOW-1: 충돌 경고를 logs_buffer뿐 아니라 타임라인 Notice로도 표시.
            let mut seen_sanitized: std::collections::HashMap<String, (usize, String)> =
                std::collections::HashMap::new();
            let mut skipped_indices: std::collections::HashSet<usize> =
                std::collections::HashSet::new();
            for (idx, mcp_cfg) in settings.mcp_servers.iter().enumerate() {
                let sanitized = Self::sanitize_tool_name_part(&mcp_cfg.name);
                if let Some((_, existing_name)) = seen_sanitized.get(&sanitized) {
                    // 충돌 감지: 이미 같은 정규화명을 가진 서버가 있음
                    // 후순위(현재 idx)만 skip
                    let warn_msg = format!(
                        "[MCP] 경고: 서버 '{}'과 '{}'이 정규화명 '{}'으로 충돌합니다. '{}' 로드를 건너뜁니다.",
                        existing_name, mcp_cfg.name, sanitized, mcp_cfg.name
                    );
                    app.state.runtime.logs_buffer.push(warn_msg.clone());
                    // [v3.3.5] 감사 LOW-1: 타임라인 Notice로도 표시하여 사용자 가시성 확보.
                    let mut block = crate::app::state::TimelineBlock::new(
                        crate::app::state::TimelineBlockKind::Notice,
                        format!("MCP 서버 '{}' 충돌 건너뜀", mcp_cfg.name),
                    );
                    block.status = crate::app::state::BlockStatus::Error;
                    block
                        .body
                        .push(crate::app::state::BlockSection::Markdown(warn_msg));
                    app.state.ui.timeline.push(block);
                    skipped_indices.insert(idx);
                } else {
                    seen_sanitized.insert(sanitized, (idx, mcp_cfg.name.clone()));
                }
            }

            for (idx, mcp_cfg) in settings.mcp_servers.iter().enumerate() {
                // [v3.3.5] index 기반 skip: 정규화 충돌 후순위만 건너뜀
                if skipped_indices.contains(&idx) {
                    continue;
                }
                let mcp_name = mcp_cfg.name.clone();
                let mcp_cmd = mcp_cfg.command.clone();
                let mcp_args = mcp_cfg.args.clone();
                let tx_mcp = tx.clone();

                tokio::spawn(async move {
                    // [v3.3.1] 감사 MEDIUM-1 수정: spawn/list_tools 실패 시 침묵하지 않고
                    // McpLoadFailed 이벤트를 전송하여 사용자에게 피드백 제공.
                    // 기존에는 if let Ok && let Ok 패턴으로 실패를 완전히 삼켰음.
                    let client = match crate::infra::mcp_client::McpClient::spawn(
                        &mcp_name, &mcp_cmd, &mcp_args,
                    )
                    .await
                    {
                        Ok(c) => c,
                        Err(e) => {
                            let _ = tx_mcp
                                .send(crate::app::event_loop::Event::Action(
                                    crate::app::action::Action::McpLoadFailed(
                                        mcp_name.clone(),
                                        format!("spawn 실패: {}", e),
                                    ),
                                ))
                                .await;
                            return;
                        }
                    };

                    let tools = match client.list_tools().await {
                        Ok(t) => t,
                        Err(e) => {
                            // [v3.3.2] 감사 HIGH-2 수정: list_tools 실패 시에도
                            // spawn된 child process를 명시적으로 kill하여 프로세스 누수 방지.
                            client.shutdown().await;
                            let _ = tx_mcp
                                .send(crate::app::event_loop::Event::Action(
                                    crate::app::action::Action::McpLoadFailed(
                                        mcp_name.clone(),
                                        format!("list_tools 실패: {}", e),
                                    ),
                                ))
                                .await;
                            return;
                        }
                    };

                    // [v2.5.3] 감사 HIGH-1: MCP 스키마를 OpenAI 호환 형식으로 래핑.
                    // MCP 서버는 {name, description, inputSchema} 반환.
                    // OpenAI API는 {type: "function", function: {name, description, parameters}} 필요.
                    // 이 래핑이 없으면 apply_dialect()도 정상 동작하지 않음.
                    //
                    // [v3.3.1] 감사 MEDIUM-2 수정: OpenAI tool name은 ^[a-zA-Z0-9_-]+$ 만 허용.
                    // 서버명/도구명에 비허용 문자(공백, 점, 슬래시 등)가 포함되면
                    // Provider가 거절하므로 정규화 함수를 적용.
                    let sanitized_server = Self::sanitize_tool_name_part(&mcp_name);
                    // [v3.3.2] 감사 HIGH-3 수정: 정규화 도구명→원본 MCP 도구명 역매핑 테이블 구성.
                    // LLM이 보내는 "mcp_{sanitized_server}_{sanitized_tool}" 형식의 이름에서
                    // 원본 MCP 도구명을 정확히 복원하기 위한 핵심 인프라.
                    let mut tool_name_map: std::collections::HashMap<String, (String, String)> =
                        std::collections::HashMap::new();
                    let schemas: Vec<serde_json::Value> = tools
                        .into_iter()
                        .filter_map(|t| {
                            let sanitized_tool = Self::sanitize_tool_name_part(&t.name);
                            // [v3.3.5] 감사 HIGH-1: OpenAI 64자 제한 준수.
                            // 서버명/도구명이 길면 truncate하여 Provider 거절 방지.
                            let mut full_name =
                                Self::build_mcp_full_name(&sanitized_server, &sanitized_tool);
                            // [v3.3.4] 감사 HIGH-1 수정: 같은 서버 내 도구명 정규화 충돌 방지.
                            // [v3.3.6] 감사 MEDIUM-2: suffix 포함 64자 초과 방어.
                            // [v3.3.7] 감사 MEDIUM-2: suffix 한계 초과 시 None 반환 (skip).
                            if tool_name_map.contains_key(&full_name) {
                                let base = full_name.clone();
                                let mut suffix = 2u32;
                                let mut resolved = false;
                                loop {
                                    let candidate = format!("{}_{}", base, suffix);
                                    if candidate.len() > Self::MAX_TOOL_NAME_LEN {
                                        let overflow = candidate.len() - Self::MAX_TOOL_NAME_LEN;
                                        let trimmed = &base[..base.len().saturating_sub(overflow)];
                                        let tc = format!("{}_{}", trimmed, suffix);
                                        if !tool_name_map.contains_key(&tc) {
                                            full_name = tc;
                                            resolved = true;
                                            break;
                                        }
                                    } else if !tool_name_map.contains_key(&candidate) {
                                        full_name = candidate;
                                        resolved = true;
                                        break;
                                    }
                                    suffix += 1;
                                    if suffix > 9999 {
                                        break;
                                    }
                                }
                                if !resolved {
                                    // suffix 한계 초과: 이 도구를 건너뜀
                                    return None;
                                }
                            }
                            // 역매핑: full_name → (sanitized_server, 원본 도구명)
                            tool_name_map.insert(
                                full_name.clone(),
                                (sanitized_server.clone(), t.name.clone()),
                            );
                            Some(serde_json::json!({
                                "type": "function",
                                "function": {
                                    "name": full_name,
                                    "description": format!("[MCP] {}", t.description),
                                    "parameters": t.input_schema
                                }
                            }))
                        })
                        .collect();
                    // [v3.3.2] 감사 HIGH-3: 정규화 서버명을 key로 전달하여
                    // mcp_clients 저장 시에도 정규화명 사용. 라우팅 일관성 보장.
                    let _ = tx_mcp
                        .send(crate::app::event_loop::Event::Action(
                            crate::app::action::Action::McpToolsLoaded(
                                sanitized_server,
                                schemas,
                                client,
                                tool_name_map,
                            ),
                        ))
                        .await;
                });
            }
        }

        app
    }

    fn check_trust_gate(&mut self) {
        if self.state.ui.is_wizard_open {
            return;
        }
        let Ok(cwd) = std::env::current_dir() else {
            return;
        };
        let root = cwd.to_string_lossy().to_string();

        if let Some(settings) = &self.state.domain.settings {
            if settings.denied_roots.contains(&root) {
                return;
            }
            if let crate::domain::settings::WorkspaceTrustState::Unknown =
                settings.get_workspace_trust(&root)
            {
                self.state.ui.trust_gate.popup = crate::app::state::TrustGatePopup::Open { root };
                self.state.ui.trust_gate.cursor_index = 0;
            }
        }
    }

    pub(crate) fn detect_workspace_root_from(
        start: &std::path::Path,
    ) -> Option<std::path::PathBuf> {
        let mut current = Some(start);
        while let Some(dir) = current {
            let has_root_marker = dir.join("Cargo.toml").exists() || dir.join(".git").exists();
            if has_root_marker {
                return Some(dir.to_path_buf());
            }
            current = dir.parent();
        }
        Some(start.to_path_buf())
    }

    fn normalize_workspace_dir() {
        if let Ok(cwd) = std::env::current_dir()
            && let Some(root) = Self::detect_workspace_root_from(&cwd)
            && root != cwd
        {
            let _ = std::env::set_current_dir(root);
        }
    }

    /// 메인 이벤트 루프. 매 틱마다 UI를 렌더링하고 이벤트를 디스패치.
    /// Input 이벤트는 handle_input으로, Action 이벤트는 handle_action으로 라우팅.
    pub async fn run(
        &mut self,
        terminal: &mut crate::tui::terminal::TerminalGuard,
        mut event_loop: EventLoop,
    ) -> Result<()> {
        loop {
            if self.state.ui.force_clear {
                let _ = terminal.clear_and_reset();
                self.state.ui.force_clear = false;
            }

            // UI 그리기
            terminal.draw(|f| {
                draw(f, &self.state);
            })?;

            // 이벤트 처리
            if let Ok(event) = event_loop.next().await {
                match event {
                    Event::Quit => {
                        // [v2.5.0] Graceful Shutdown: 모든 활성 도구 실행 취소
                        for token in self.state.runtime.active_tool_cancel_tokens.values() {
                            token.cancel();
                        }
                        self.state.runtime.active_tool_cancel_tokens.clear();
                        self.state.should_quit = true;
                    }
                    Event::Resize(_, _) => {
                        // [v1.5.0] 터미널 리사이즈 처리
                        let _ = terminal.autoresize();
                    }
                    Event::Input(key) => {
                        self.handle_input(key);
                        self.sync_toolbar();
                    }
                    Event::Action(action) => {
                        self.handle_action(action);
                        self.sync_toolbar();
                    }
                    Event::Tick => {
                        // [v0.1.0-beta.18] 애니메이션 틱 카운터 증가
                        self.state.ui.tick_count = self.state.ui.tick_count.wrapping_add(1);
                        self.sync_toolbar();
                        self.expire_pending_approval_if_needed(Self::unix_time_ms());

                        // [v2.3.0] Phase 31: 토스트 알림 만료 체크
                        if let Some(toast) = &self.state.ui.toast
                            && std::time::Instant::now() >= toast.expires_at
                        {
                            self.state.ui.toast = None;
                        }

                        // [Phase 15-E] follow_tail일 때 커서를 최신 블록으로 동기화
                        if self.state.ui.timeline_follow_tail && !self.state.ui.timeline.is_empty()
                        {
                            self.state.ui.timeline_cursor = self.state.ui.timeline.len() - 1;
                        }

                        // 컨텍스트 자동 압축 트리거
                        if self.state.domain.session.needs_auto_compaction {
                            self.state.domain.session.needs_auto_compaction = false;
                            self.handle_slash_command("/compact");
                        }
                    }
                    Event::Mouse(mouse_event) => {
                        self.handle_mouse(mouse_event);
                    }
                }
            }

            if self.state.should_quit {
                break;
            }
        }

        // [v3.3.1] 감사 HIGH-1 수정: 앱 종료 시 MCP 서버 자식 프로세스 명시적 kill.
        // Event::Quit, /quit, Ctrl-C, SIGTERM 모든 종료 경로가 이 지점을 통과하므로
        // 여기서 한 번만 shutdown()을 호출하면 프로세스 누수를 완전히 방지할 수 있다.
        for client in self.state.runtime.mcp_clients.values() {
            client.shutdown().await;
        }
        self.state.runtime.mcp_clients.clear();

        Ok(())
    }

    fn unix_time_ms() -> u64 {
        std::time::UNIX_EPOCH
            .elapsed()
            .unwrap_or_default()
            .as_millis() as u64
    }

    fn refresh_repo_map_if_needed(&mut self, force: bool) {
        if force {
            self.state.runtime.repo_map.mark_stale();
        }
        if !self.state.runtime.repo_map.begin_refresh() {
            return;
        }

        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string());
        let tx = self.action_tx.clone();
        tokio::spawn(async move {
            match crate::domain::repo_map::generate_repo_map_async(cwd).await {
                Ok(repo_map) => {
                    let _ = tx
                        .send(Event::Action(action::Action::RepoMapReady(repo_map)))
                        .await;
                }
                Err(err) => {
                    let _ = tx
                        .send(Event::Action(action::Action::RepoMapFailed(
                            err.to_string(),
                        )))
                        .await;
                }
            }
        });
    }

    fn mark_repo_map_stale(&mut self) {
        self.state.runtime.repo_map.mark_stale();
    }

    fn tool_requires_repo_map_refresh(tool_name: &str) -> bool {
        matches!(tool_name, "WriteFile" | "ReplaceFileContent" | "ExecShell")
    }

    pub(crate) fn expire_pending_approval_if_needed(&mut self, now_ms: u64) -> bool {
        let Some(started_at) = self.state.runtime.approval.pending_since_ms else {
            return false;
        };
        if self.state.runtime.approval.pending_tool.is_none()
            || now_ms.saturating_sub(started_at) < Self::APPROVAL_TTL_MS
        {
            return false;
        }

        let tool = self
            .state
            .runtime
            .approval
            .pending_tool
            .take()
            .unwrap_or_else(|| crate::domain::tool_result::ToolCall {
                name: String::new(),
                args: serde_json::json!({}),
            });
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

        for block in self.state.ui.timeline.iter_mut().rev() {
            if block.kind == crate::app::state::TimelineBlockKind::Approval
                && block.status == crate::app::state::BlockStatus::NeedsApproval
            {
                block.status = crate::app::state::BlockStatus::Error;
                block.body.push(crate::app::state::BlockSection::Markdown(
                    "승인 대기 시간이 초과되어 요청이 자동 취소되었습니다.".to_string(),
                ));
                break;
            }
        }

        let res = crate::domain::tool_result::ToolResult {
            tool_name: tool.name.clone(),
            stdout: String::new(),
            stderr: "승인 응답 시간 초과로 도구 실행이 취소되었습니다.".to_string(),
            exit_code: 1,
            is_error: true,
            tool_call_id: tool_call_id.clone(),
            is_truncated: false,
            original_size_bytes: None,
            affected_paths: vec![],
        };
        let mut block = crate::app::state::TimelineBlock::new(
            crate::app::state::TimelineBlockKind::Notice,
            "승인 요청 시간 초과",
        );
        block.status = crate::app::state::BlockStatus::Error;
        block.body.push(crate::app::state::BlockSection::Markdown(
            "승인 대기 시간이 초과되어 자동 취소되었습니다.".to_string(),
        ));
        self.state.ui.timeline.push(block);

        let _ = self
            .action_tx
            .try_send(crate::app::event_loop::Event::Action(
                crate::app::action::Action::ToolFinished(Box::new(res), tool_index),
            ));

        // Pop next queued approval if any
        if let Some((next_tool, next_id, next_idx)) =
            self.state.runtime.approval.queued_approvals.pop_front()
        {
            self.state.runtime.approval.pending_tool = Some(next_tool.clone());
            self.state.runtime.approval.pending_tool_call_id = next_id.clone();
            self.state.runtime.approval.pending_tool_index = Some(next_idx);
            self.state.runtime.approval.pending_since_ms = Some(now_ms);
            if let Some(registry_tool) =
                crate::tools::registry::GLOBAL_REGISTRY.get_tool(&next_tool.name)
            {
                self.state.runtime.approval.diff_preview =
                    registry_tool.generate_diff_preview(&next_tool.args);
            }
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
        }

        true
    }

    /// 비동기 액션 이벤트를 각 도메인별 핸들러로 라우팅.
    /// 도구 결과, 채팅 응답, 모델 목록, 인증 결과, 컨텍스트 요약 등 처리.
    fn summarize_failure_detail(detail: &str) -> String {
        Self::preserve_failure_edges(detail, Self::AUTO_VERIFY_MODEL_DETAIL_CHARS)
    }

    fn summarize_failure_notice(detail: &str) -> String {
        let collapsed = detail.replace('\n', " / ");
        let trimmed = collapsed.trim();
        if trimmed.chars().count() > Self::AUTO_VERIFY_NOTICE_CHARS {
            format!(
                "{}...",
                trimmed
                    .chars()
                    .take(Self::AUTO_VERIFY_NOTICE_CHARS)
                    .collect::<String>()
            )
        } else {
            trimmed.to_string()
        }
    }

    fn preserve_failure_edges(detail: &str, max_chars: usize) -> String {
        let trimmed = detail.trim();
        let chars: Vec<char> = trimmed.chars().collect();
        if chars.len() <= max_chars {
            return trimmed.to_string();
        }

        let head_len = max_chars / 2;
        let tail_len = max_chars.saturating_sub(head_len + 5);
        let head: String = chars.iter().take(head_len).collect();
        let tail_start = chars.len().saturating_sub(tail_len);
        let tail: String = chars.iter().skip(tail_start).collect();
        format!("{head}\n...\n{tail}")
    }

    /// [v0.1.0-beta.26] 자가 치유용 실패 컨텍스트는 UI 요약보다 훨씬 풍부해야 한다.
    /// stderr를 우선 보존하되 stdout의 후반부 단서도 함께 전달하여 무의미한 재시도를 줄인다.
    pub(crate) fn build_auto_verify_failure_context(
        res: &crate::domain::tool_result::ToolResult,
    ) -> String {
        let mut sections = vec![
            format!("Tool: {}", res.tool_name),
            format!("Exit Code: {}", res.exit_code),
        ];

        if !res.stderr.trim().is_empty() {
            sections.push(format!(
                "STDERR:\n{}",
                Self::preserve_failure_edges(&res.stderr, 1200)
            ));
        }
        if !res.stdout.trim().is_empty() {
            sections.push(format!(
                "STDOUT:\n{}",
                Self::preserve_failure_edges(&res.stdout, 800)
            ));
        }

        sections.join("\n")
    }

    fn push_auto_verify_notice(
        &mut self,
        title: &str,
        status: crate::app::state::BlockStatus,
        body: String,
    ) {
        let mut block = crate::app::state::TimelineBlock::new(
            crate::app::state::TimelineBlockKind::Notice,
            title,
        )
        .with_depth(1);
        block.status = status;
        block
            .body
            .push(crate::app::state::BlockSection::Markdown(body));
        self.state.ui.timeline.push(block);
    }

    /// [v0.1.0-beta.25] 도구 실패 시 Auto-Verify 상태 머신을 실제로 전진시킨다.
    /// 반환값이 true면 후속 LLM 재전송을 수행하고, false면 사용자 수동 개입으로 종료한다.
    pub(crate) fn advance_auto_verify_after_failure(&mut self, failure_detail: &str) -> bool {
        let failure_excerpt = Self::summarize_failure_detail(failure_detail);
        let notice_excerpt = Self::summarize_failure_notice(failure_detail);
        let next_retry = match self.state.runtime.auto_verify {
            crate::app::state::AutoVerifyState::Idle => 1,
            crate::app::state::AutoVerifyState::Healing { retries } => retries + 1,
            // [v2.5.0] 이미 Aborted 상태에서 다른 병렬 도구가 실패해도 즉시 차단 유지
            crate::app::state::AutoVerifyState::Aborted => return false,
        };

        if next_retry >= Self::MAX_AUTO_VERIFY_RETRIES {
            // [v2.5.0] Aborted 상태로 전환: 병렬 도구의 최종 flush 시점까지 LLM 재전송 차단
            self.state.runtime.auto_verify = crate::app::state::AutoVerifyState::Aborted;
            let abort_message = format!(
                "자동 복구가 {}/{} 실패하여 중단되었습니다. 수동 개입이 필요합니다.\n마지막 오류: {}",
                next_retry,
                Self::MAX_AUTO_VERIFY_RETRIES,
                notice_excerpt
            );
            self.state
                .domain
                .session
                .add_message(crate::providers::types::ChatMessage {
                    role: crate::providers::types::Role::System,
                    content: Some(format!(
                        "[Auto-Verify: Abort]\nAuto-healing stopped after {} failed attempts.\nManual intervention is required.\nLast failure: {}",
                        next_retry, failure_excerpt
                    )),
                    tool_calls: None,
                    tool_call_id: None,
                    pinned: false,
                });
            self.push_auto_verify_notice(
                "[Auto-Verify: Abort]",
                crate::app::state::BlockStatus::Error,
                abort_message,
            );
            return false;
        }

        self.state.runtime.auto_verify = crate::app::state::AutoVerifyState::Healing {
            retries: next_retry,
        };
        self.state
            .domain
            .session
            .add_message(crate::providers::types::ChatMessage {
                role: crate::providers::types::Role::System,
                content: Some(format!(
                    "[Auto-Verify: Healing {}/{}]\nThe previous tool execution failed.\nAnalyze the failure, choose the safest next step, and call at most one tool if needed.\nPrefer read-only inspection before another write or shell command.\nFailure detail: {}",
                    next_retry,
                    Self::MAX_AUTO_VERIFY_RETRIES,
                    failure_excerpt
                )),
                tool_calls: None,
                tool_call_id: None,
                pinned: false,
            });
        self.push_auto_verify_notice(
            "[Auto-Verify: Healing]",
            crate::app::state::BlockStatus::Running,
            format!(
                "도구 실행이 실패하여 자동 복구를 시도합니다 ({}/{}).\n오류 요약: {}",
                next_retry,
                Self::MAX_AUTO_VERIFY_RETRIES,
                notice_excerpt
            ),
        );
        true
    }

    pub(crate) fn reset_auto_verify_after_success(&mut self) {
        if matches!(
            self.state.runtime.auto_verify,
            crate::app::state::AutoVerifyState::Healing { .. }
        ) {
            self.state.runtime.auto_verify = crate::app::state::AutoVerifyState::Idle;
            self.state
                .runtime
                .logs_buffer
                .push("[Auto-Verify] 복구 루프를 정상 종료했습니다.".to_string());
        }
    }

    fn scroll_timeline_up(&mut self, lines: u16) {
        self.state.ui.timeline_scroll = self.state.ui.timeline_scroll.saturating_add(lines);
        self.state.ui.timeline_follow_tail = false;
    }

    fn flush_stream_accumulator(&mut self) {
        let mut trailing = std::mem::take(&mut self.state.runtime.streaming_masker.trailing_buffer);
        if !trailing.is_empty() {
            if let Some(re) = &self.state.runtime.secret_mask_regex {
                let masked = re.replace_all(&trailing, "[REDACTED]").to_string();
                trailing = masked;
            }
            self.state.runtime.stream_accumulator.push_str(&trailing);
        }

        let mut rest = String::new();
        std::mem::swap(&mut self.state.runtime.stream_accumulator, &mut rest);
        if !rest.is_empty() {
            if rest.ends_with('\n') {
                rest.pop();
            }
            if rest.ends_with('\r') {
                rest.pop();
            }
            self.state.runtime.logs_buffer.push(rest);
        }
    }

    fn scroll_timeline_down(&mut self, lines: u16) {
        self.state.ui.timeline_scroll = self.state.ui.timeline_scroll.saturating_sub(lines);
        if self.state.ui.timeline_scroll == 0 {
            self.state.ui.timeline_follow_tail = true;
        }
    }

    fn mouse_target(
        &self,
        mouse: crossterm::event::MouseEvent,
        term_cols: u16,
        term_rows: u16,
    ) -> MousePaneTarget {
        if term_rows < 5 {
            return MousePaneTarget::Other;
        }

        let main_top = 1;
        let composer_top = term_rows.saturating_sub(3);
        let status_row = composer_top.saturating_sub(1);
        if mouse.row >= composer_top {
            return MousePaneTarget::Composer;
        }
        if mouse.row < main_top || mouse.row == status_row {
            return MousePaneTarget::Other;
        }

        let inspector_active =
            self.state.ui.show_inspector || self.state.runtime.approval.pending_tool.is_some();
        if !inspector_active {
            return MousePaneTarget::Timeline;
        }

        let inspector_width = (term_cols as f32 * 0.30).clamp(32.0, 48.0) as u16;
        let timeline_width = term_cols.saturating_sub(inspector_width).max(72);
        if mouse.column >= timeline_width {
            MousePaneTarget::Inspector
        } else {
            MousePaneTarget::Timeline
        }
    }

    pub(crate) fn handle_action(&mut self, action: action::Action) {
        match action {
            action::Action::SubmitChatRequest(final_text) => {
                self.submit_chat_request(final_text);
            }
            action::Action::AddTimelineNotice(msg) => {
                self.state
                    .ui
                    .timeline
                    .push(crate::app::state::TimelineBlock::new(
                        crate::app::state::TimelineBlockKind::Notice,
                        msg,
                    ));
            }
            action::Action::ChatStarted => {
                // thinking indicator 시작 + 블록 상태 갱신
                self.state.runtime.is_thinking = true;
                if let Some(idx) = self.state.runtime.active_chat_block_idx
                    && let Some(block) = self.state.ui.timeline.get_mut(idx)
                {
                    block.status = crate::app::state::BlockStatus::Running;
                    // [v0.1.0-beta.26] send_chat_message에서 이미 빈 Markdown을 넣었으므로 여기서 중복으로 넣지 않음.
                    if block.body.is_empty() {
                        block
                            .body
                            .push(crate::app::state::BlockSection::Markdown(String::new()));
                    }
                }
            }

            action::Action::ChatDelta(token) => {
                // SSE 토큰 수신: 스트리밍 중간 결과에 append
                // [v0.1.0-beta.26] ChatDelta에서 is_thinking을 false로 만들면 중간에 새 채팅 요청이 들어올 수 있으므로 제거함.
                if let Some(idx) = self.state.runtime.active_chat_block_idx
                    && let Some(block) = self.state.ui.timeline.get_mut(idx)
                    && let Some(crate::app::state::BlockSection::Markdown(buf)) =
                        block.body.last_mut()
                {
                    buf.push_str(&token);
                }
            }

            action::Action::ChatResponseOk(res) => {
                // [v0.1.0-beta.16] 추론 완료: thinking indicator 비활성화
                self.state.runtime.is_thinking = false;
                // 토큰 예산 갱신
                self.state.domain.session.token_budget_used += res.input_tokens + res.output_tokens;
                self.state.domain.session.add_message(res.message.clone());

                // [v0.1.0-beta.20] AI 응답을 JSONL 세션 로그에 동기 기록
                // [v0.1.0-beta.18→20 수정] 비동기 append_message를 동기 API로 교체.
                if let Some(ref logger) = self.state.domain.session_logger
                    && let Err(e) = logger.append_message(&res.message)
                {
                    self.state
                        .runtime
                        .logs_buffer
                        .push(format!("[SessionLog] AI 응답 기록 실패: {}", e));
                }

                // 스트리밍 Delta가 있으면 완성된 메시지로 변환
                if let Some(idx) = self.state.runtime.active_chat_block_idx
                    && let Some(block) = self.state.ui.timeline.get_mut(idx)
                {
                    block.status = crate::app::state::BlockStatus::Done;
                    if let Some(crate::app::state::BlockSection::Markdown(buf)) =
                        block.body.last_mut()
                    {
                        *buf = res.message.content.clone().unwrap_or_default();
                    } else {
                        block.body.push(crate::app::state::BlockSection::Markdown(
                            res.message.content.clone().unwrap_or_default(),
                        ));
                    }
                }

                // [v0.1.0-beta.22] assistant_turn_count 삭제됨 (재감사 6차).
                // 사유: 첫 턴 차단 로직이 제거되어 카운터만 증가하는 데드 코드였음.

                // 응답에서 도구 호출 감지 및 처리 (tool_runtime.rs에 위임)
                let has_tool_calls = res
                    .message
                    .tool_calls
                    .as_ref()
                    .is_some_and(|calls| !calls.is_empty());
                self.process_tool_calls_from_response(&res.message);
                if matches!(
                    self.state.runtime.auto_verify,
                    crate::app::state::AutoVerifyState::Healing { .. }
                ) && !has_tool_calls
                {
                    self.state.runtime.auto_verify = crate::app::state::AutoVerifyState::Idle;
                    self.state.runtime.logs_buffer.push(
                        "[Auto-Verify] 모델이 도구 재시도 없이 설명 응답으로 복구 루프를 종료했습니다."
                            .to_string(),
                    );
                }
            }

            action::Action::ChatResponseErr(e) => {
                // [v0.1.0-beta.16] 추론 완료: thinking indicator 비활성화
                self.state.runtime.is_thinking = false;

                // 스트리밍 Delta가 있으면 에러 메시지로 변환
                if let Some(idx) = self.state.runtime.active_chat_block_idx
                    && let Some(block) = self.state.ui.timeline.get_mut(idx)
                {
                    block.status = crate::app::state::BlockStatus::Error;
                    block
                        .body
                        .push(crate::app::state::BlockSection::Markdown(format!(
                            "\n\nProvider Error: {}",
                            e.to_actionable()
                        )));
                } else {
                    let mut block = crate::app::state::TimelineBlock::new(
                        crate::app::state::TimelineBlockKind::Notice,
                        "Provider Error",
                    );
                    block.status = crate::app::state::BlockStatus::Error;
                    block.body.push(crate::app::state::BlockSection::Markdown(
                        e.to_actionable().to_string(),
                    ));
                    self.state.ui.timeline.push(block);
                }

                self.state
                    .domain
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: Some(format!("Provider Error: {}", e.to_actionable())),
                        tool_calls: None,
                        tool_call_id: None,
                        pinned: false,
                    });
            }

            // === [v0.1.0-beta.18] 도구 라이프사이클 ===
            action::Action::ToolQueued(ref tool_call, _) => {
                // 타임라인에 "대기중" 카드 추가
                let tool_name = format!("{:?}", tool_call)
                    .chars()
                    .take(30)
                    .collect::<String>();
                let mut block = crate::app::state::TimelineBlock::new(
                    crate::app::state::TimelineBlockKind::ToolRun,
                    tool_name,
                )
                .with_depth(1);
                block.status = crate::app::state::BlockStatus::Idle;
                block.body.push(crate::app::state::BlockSection::Markdown(
                    "권한 검사 중...".to_string(),
                ));
                self.state.ui.timeline.push(block);
            }

            action::Action::ToolStarted(name) => {
                // 마지막 ToolRun 블록의 상태를 Running으로 갱신
                for block in self.state.ui.timeline.iter_mut().rev() {
                    if block.kind == crate::app::state::TimelineBlockKind::ToolRun
                        && (block.title == name
                            || block.status == crate::app::state::BlockStatus::Idle)
                    {
                        block.status = crate::app::state::BlockStatus::Running;
                        break;
                    }
                }
            }

            action::Action::ToolOutputChunk(mut chunk) => {
                self.mask_secrets(&mut chunk);
                self.state.runtime.stream_accumulator.push_str(&chunk);
                let mut new_lines: i32 = 0;

                while let Some(idx) = self.state.runtime.stream_accumulator.find('\n') {
                    let mut rest = self.state.runtime.stream_accumulator.split_off(idx + 1);
                    std::mem::swap(&mut self.state.runtime.stream_accumulator, &mut rest);

                    let mut line = rest;
                    if line.ends_with('\n') {
                        line.pop();
                    }
                    if line.ends_with('\r') {
                        line.pop();
                    }

                    self.state.runtime.logs_buffer.push(line);
                    new_lines += 1;
                }

                if new_lines == 0 {
                    return; // 라인이 추가되지 않았으므로 스크롤 로직 생략
                }

                // [v1.2.0] OOM 방지 및 Pruning 시 Sticky Scroll Sync
                let max_lines = 10000;
                let mut trimmed = 0;
                if self.state.runtime.logs_buffer.len() > max_lines {
                    trimmed = self.state.runtime.logs_buffer.len() - max_lines;
                    self.state.runtime.logs_buffer.drain(0..trimmed);
                }

                if !self.state.ui.timeline_follow_tail {
                    let mut current_scroll = self.state.ui.inspector_scroll.get() as i32;
                    // 새 라인이 new_lines개 추가되었으므로 offset +new_lines.
                    // 상단이 trimmed개 지워졌으므로 offset -trimmed.
                    current_scroll += new_lines - (trimmed as i32);

                    if current_scroll < 0 {
                        // 보고 있던 라인이 삭제된 경우, 새로운 상단(가장 큰 값)에 위치시킴
                        self.state.ui.inspector_scroll.set(u16::MAX);
                    } else {
                        self.state.ui.inspector_scroll.set(current_scroll as u16);
                    }
                } else {
                    self.state.ui.inspector_scroll.set(0);
                }
            }

            action::Action::ToolFinished(mut res, tool_index) => {
                self.flush_stream_accumulator();
                // [v2.5.0] 완료된 도구의 취소 토큰을 맵에서 제거
                let token_key = res
                    .tool_call_id
                    .clone()
                    .unwrap_or_else(|| format!("tool_{}", tool_index));
                self.state
                    .runtime
                    .active_tool_cancel_tokens
                    .remove(&token_key);

                // [v1.9.0] Phase 27: 터미널 타이틀 & 작업표시줄 진행률 복구 (OSC)
                {
                    use std::io::Write;
                    let title_reset = "\x1b]0;smlcli\x07";
                    let progress_reset = "\x1b]9;4;0;0\x07";
                    print!("{}{}", title_reset, progress_reset);
                    let _ = std::io::stdout().flush();
                }

                // [v1.6.0] RepoMap Dirty Flag 갱신
                // [v2.5.0] is_write_tool()과 대상이 다름: GitCheckpoint는 스냅샷 보존(git stash/commit)
                // 도구로 파일 내용을 변경하지 않으므로 RepoMap 갱신 대상에서 의도적으로 제외.
                if matches!(
                    res.tool_name.as_str(),
                    "WriteFile" | "ReplaceFileContent" | "DeleteFile" | "ExecShell"
                ) && !res.is_error
                {
                    self.state.runtime.repo_map_dirty = true;
                }

                // [v1.6.0] 마스킹
                self.mask_secrets(&mut res.stdout);
                self.mask_secrets(&mut res.stderr);

                // 결과를 보류 목록에 저장
                self.state.runtime.pending_tool_outcomes.push((
                    tool_index,
                    crate::app::state::ToolOutcome::Success(res.clone()),
                ));

                // 타임라인 업데이트 (보류 없이 즉시 반영)

                // [UX] ExecShell 프로세스 복귀 후 터미널 잔상(Ghosting) 제거
                if res.tool_name == "ExecShell" {
                    self.state.ui.force_clear = true;
                }

                // 타임라인 ToolRun 상태를 Done/Error로 갱신
                let final_status = if res.is_error {
                    crate::app::state::BlockStatus::Error
                } else {
                    crate::app::state::BlockStatus::Done
                };
                let summary = Self::generate_tool_summary(&res);

                // [Phase 16] Diff Summary Calculation
                let mut diff_summary = None;
                let mut display_mode = crate::app::state::BlockDisplayMode::Expanded;
                if res.tool_name == "ReplaceFileContent" || res.tool_name == "WriteFile" {
                    let additions = res
                        .stdout
                        .lines()
                        .filter(|l| l.starts_with("+ ") && !l.starts_with("+++"))
                        .count();
                    let deletions = res
                        .stdout
                        .lines()
                        .filter(|l| l.starts_with("- ") && !l.starts_with("---"))
                        .count();
                    if additions + deletions > 10 {
                        display_mode = crate::app::state::BlockDisplayMode::Collapsed;
                    }
                    if additions > 0 || deletions > 0 {
                        diff_summary = Some((additions, deletions));
                    }
                }

                for block in self.state.ui.timeline.iter_mut().rev() {
                    if block.kind == crate::app::state::TimelineBlockKind::ToolRun
                        && block.tool_call_id == res.tool_call_id
                        && (block.status == crate::app::state::BlockStatus::Running
                            || block.status == crate::app::state::BlockStatus::Idle)
                    {
                        block.status = final_status.clone();
                        if diff_summary.is_some() {
                            block.display_mode = display_mode.clone();
                            block.diff_summary = diff_summary;
                        }
                        block
                            .body
                            .push(crate::app::state::BlockSection::ToolSummary {
                                tool_name: res.tool_name.clone(),
                                summary: summary.clone(),
                            });
                        break;
                    }
                }

                if res.is_error {
                    let failure_context = Self::build_auto_verify_failure_context(&res);
                    self.mark_repo_map_stale();
                    self.refresh_repo_map_if_needed(false);
                    // abort 시 Aborted 상태가 runtime에 지속 저장됨
                    let _ = self.advance_auto_verify_after_failure(&failure_context);
                } else {
                    if Self::tool_requires_repo_map_refresh(&res.tool_name) {
                        self.mark_repo_map_stale();
                        self.refresh_repo_map_if_needed(false);
                    }
                    self.reset_auto_verify_after_success();

                    // [v3.0.0] Phase 40: Git-Native Integration 자동 커밋
                    if let Some(settings) = &self.state.domain.settings {
                        let should_commit = settings.git_integration.auto_commit
                            && settings
                                .git_integration
                                .commit_tools
                                .contains(&res.tool_name);

                        // [v2.5.1] 감사 HIGH-1 수정: affected_paths가 비어있으면 commit skip.
                        // 실제 변경된 파일만 stage하여 사용자 WIP를 보호.
                        if should_commit && !res.affected_paths.is_empty() {
                            let cwd = std::env::current_dir()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_else(|_| ".".to_string());

                            let file_refs: Vec<&str> =
                                res.affected_paths.iter().map(|s| s.as_str()).collect();
                            match crate::infra::git_engine::GitEngine::auto_commit(
                                &cwd,
                                &res.tool_name,
                                &file_refs,
                                &settings.git_integration.commit_prefix,
                            ) {
                                Ok(msg) => {
                                    if msg != "No changes to commit" {
                                        self.state.ui.timeline.push(
                                            crate::app::state::TimelineBlock {
                                                id: uuid::Uuid::new_v4().to_string(),
                                                kind:
                                                    crate::app::state::TimelineBlockKind::GitCommit,
                                                status: crate::app::state::BlockStatus::Done,
                                                role: None,
                                                title: msg.clone(),
                                                subtitle: None,
                                                body: vec![
                                                    crate::app::state::BlockSection::Markdown(
                                                        "Auto-commit successful.".to_string(),
                                                    ),
                                                ],
                                                tool_call_id: None,
                                                depth: 0,
                                                display_mode:
                                                    crate::app::state::BlockDisplayMode::Expanded,
                                                diff_summary: None,
                                                created_at_ms: std::time::SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap_or_default()
                                                    .as_millis()
                                                    as u64,
                                                pinned: false,
                                            },
                                        );
                                        self.state.ui.timeline_scroll = 0;
                                        self.state.ui.timeline_follow_tail = true;
                                    }
                                }
                                Err(e) => {
                                    self.state
                                        .ui
                                        .timeline
                                        .push(crate::app::state::TimelineBlock {
                                            id: uuid::Uuid::new_v4().to_string(),
                                            kind: crate::app::state::TimelineBlockKind::Notice,
                                            status: crate::app::state::BlockStatus::Error,
                                            role: None,
                                            title: "Auto Commit Failed".to_string(),
                                            subtitle: None,
                                            body: vec![crate::app::state::BlockSection::Markdown(
                                                format!("Git 자동 커밋 중 오류 발생:\n{}", e),
                                            )],
                                            tool_call_id: None,
                                            depth: 0,
                                            display_mode:
                                                crate::app::state::BlockDisplayMode::Expanded,
                                            diff_summary: None,
                                            created_at_ms: std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap_or_default()
                                                .as_millis()
                                                as u64,
                                            pinned: false,
                                        });
                                }
                            }
                        }
                    }
                }

                if Self::is_write_tool(&res.tool_name) {
                    self.state.runtime.is_write_tool_running = false;
                    if let Some((next_tool, next_id, next_idx)) =
                        self.state.runtime.write_tool_queue.pop_front()
                    {
                        self.state.runtime.is_write_tool_running = true;
                        self.execute_tool_async(next_tool, next_id, next_idx);
                    }
                }

                self.state.runtime.pending_tool_executions =
                    self.state.runtime.pending_tool_executions.saturating_sub(1);

                if self.state.runtime.pending_tool_executions == 0 {
                    self.flush_pending_tool_outcomes();
                    // [v2.5.0] Aborted 상태가 runtime에 지속되므로, 어떤 도구가 마지막이든
                    // abort 결정이 일관되게 반영됨
                    if self.state.runtime.auto_verify != crate::app::state::AutoVerifyState::Aborted
                        && self.state.runtime.approval.pending_tool.is_none()
                        && self.state.runtime.approval.queued_approvals.is_empty()
                    {
                        // [v3.7.1] 직접 실행(!)된 도구는 LLM 자동 전송을 건너뜀
                        if res.tool_call_id.is_some() {
                            self.send_chat_message_internal();
                        }
                    }
                    // Aborted 상태를 Idle로 리셋 (다음 사용자 입력 대기)
                    if self.state.runtime.auto_verify == crate::app::state::AutoVerifyState::Aborted
                    {
                        self.state.runtime.auto_verify = crate::app::state::AutoVerifyState::Idle;
                    }
                }
            }

            action::Action::ToolSummaryReady(new_summary) => {
                // 외부에서 생성된 요약으로 마지막 ToolRun 갱신
                for block in self.state.ui.timeline.iter_mut().rev() {
                    if block.kind == crate::app::state::TimelineBlockKind::ToolRun
                        && let Some(crate::app::state::BlockSection::ToolSummary {
                            summary, ..
                        }) = block.body.last_mut()
                    {
                        *summary = new_summary;
                        break;
                    }
                }
            }

            action::Action::ToolError(e, tool_call_id, tool_index) => {
                self.flush_stream_accumulator();
                // [v2.5.0] 오류 발생한 도구의 취소 토큰을 맵에서 제거
                let token_key = tool_call_id
                    .clone()
                    .unwrap_or_else(|| format!("tool_{}", tool_index));
                self.state
                    .runtime
                    .active_tool_cancel_tokens
                    .remove(&token_key);

                // [v1.9.0] Phase 27: 터미널 타이틀 & 작업표시줄 진행률 복구 (OSC)
                {
                    use std::io::Write;
                    let title_reset = "\x1b]0;smlcli\x07";
                    let progress_reset = "\x1b]9;4;0;0\x07";
                    print!("{}{}", title_reset, progress_reset);
                    let _ = std::io::stdout().flush();
                }
                let mut failure_detail = e.to_actionable().to_string();
                self.mask_secrets(&mut failure_detail);

                // 결과를 보류 목록에 저장
                self.state.runtime.pending_tool_outcomes.push((
                    tool_index,
                    crate::app::state::ToolOutcome::Error(e, tool_call_id.clone()),
                ));

                self.mark_repo_map_stale();
                self.refresh_repo_map_if_needed(false);

                // 타임라인 ToolRun 상태를 Error로 갱신
                for block in self.state.ui.timeline.iter_mut().rev() {
                    if block.kind == crate::app::state::TimelineBlockKind::ToolRun
                        && block.tool_call_id == tool_call_id
                        && (block.status == crate::app::state::BlockStatus::Running
                            || block.status == crate::app::state::BlockStatus::Idle)
                    {
                        block.status = crate::app::state::BlockStatus::Error;
                        block
                            .body
                            .push(crate::app::state::BlockSection::Markdown(format!(
                                "실패: {}",
                                failure_detail.chars().take(80).collect::<String>()
                            )));
                        break;
                    }
                }

                // abort 시 Aborted 상태가 runtime에 지속 저장됨
                let _ = self.advance_auto_verify_after_failure(&failure_detail);

                self.state.runtime.pending_tool_executions =
                    self.state.runtime.pending_tool_executions.saturating_sub(1);

                if self.state.runtime.pending_tool_executions == 0 {
                    self.flush_pending_tool_outcomes();
                    // [v2.5.0] Aborted 상태가 runtime에 지속되므로 병렬 도구 간 일관성 보장
                    if self.state.runtime.auto_verify != crate::app::state::AutoVerifyState::Aborted
                        && self.state.runtime.approval.pending_tool.is_none()
                        && self.state.runtime.approval.queued_approvals.is_empty()
                        && tool_call_id.is_some()
                    {
                        self.send_chat_message_internal();
                    }
                    // Aborted 상태를 Idle로 리셋 (다음 사용자 입력 대기)
                    if self.state.runtime.auto_verify == crate::app::state::AutoVerifyState::Aborted
                    {
                        self.state.runtime.auto_verify = crate::app::state::AutoVerifyState::Idle;
                    }
                }
            }

            // === 기존 유지 ===
            action::Action::ModelsFetched(res, source) => {
                self.handle_models_fetched(res, source);
            }
            action::Action::CredentialValidated(res) => {
                self.handle_credential_validated(res);
            }
            action::Action::WizardSaveFinished(res) => {
                self.state.ui.wizard.is_loading_models = false;
                match res {
                    Ok(_) => {
                        crate::providers::registry::reload_providers();
                        self.state.ui.is_wizard_open = false;
                    }
                    Err(e) => {
                        self.state.ui.wizard.step = crate::app::state::WizardStep::Saving;
                        self.state.ui.wizard.err_msg = Some(format!("설정 저장 실패: {}", e));
                    }
                }
            }
            action::Action::ConfigSaveFinished(res) => {
                if let Err(e) = res {
                    let mut block = crate::app::state::TimelineBlock::new(
                        crate::app::state::TimelineBlockKind::Notice,
                        "설정 저장 실패",
                    );
                    block.status = crate::app::state::BlockStatus::Error;
                    block.body.push(crate::app::state::BlockSection::Markdown(format!(
                        "디스크 용량 부족 또는 권한 문제로 설정을 저장하지 못했습니다.\n\n상세 오류:\n{}",
                        e
                    )));
                    self.state.ui.timeline.push(block);
                }
            }
            action::Action::RepoMapReady(repo_map) => {
                self.state.runtime.repo_map.finish_success(repo_map);
            }
            action::Action::RepoMapFailed(err) => {
                self.state.runtime.repo_map.finish_error(err.clone());
                self.state
                    .runtime
                    .logs_buffer
                    .push(format!("[Repo Map] 갱신 실패: {}", err));
            }
            action::Action::ContextSummaryOk(summary) => {
                self.state.domain.session.apply_summary(&summary);
                let mut block = crate::app::state::TimelineBlock::new(
                    crate::app::state::TimelineBlockKind::Notice,
                    "컨텍스트 압축 완료",
                );
                block
                    .body
                    .push(crate::app::state::BlockSection::Markdown(summary));
                self.state.ui.timeline.push(block);
            }
            action::Action::ContextSummaryErr(e) => {
                self.state
                    .domain
                    .session
                    .apply_summary(&format!("Fallback due to error: {}", e));
            }
            action::Action::SilentHealthCheckFailed => {
                self.state.ui.toast = Some(crate::app::state::ToastNotification {
                    message: "⚠ smlcli doctor를 실행하여 시스템 환경을 확인하세요.".to_string(),
                    is_error: true,
                    expires_at: std::time::Instant::now() + std::time::Duration::from_secs(5),
                });
            }
            action::Action::McpToolsLoaded(name, mut schemas, client, tool_name_map) => {
                // [v3.3.2] 감사 HIGH-3 수정: 정규화된 서버명을 key로 저장.
                // 스키마에서 사용하는 정규화명과 일치하여 라우팅 정합성 보장.
                self.state.runtime.mcp_clients.insert(name.clone(), client);
                // [v3.3.7] 감사 HIGH-1 수정: schemas를 먼저 push하지 않고,
                // 전역 충돌 해소 후 schema.function.name까지 수정한 뒤 push.
                // 이전: schemas 먼저 push → map key만 suffix 변경 → schema name ≠ map key.
                // 수정: tool_name_map 순회 시 충돌이면 schema의 name도 함께 변경.
                // [v3.3.7] 감사 MEDIUM-2: suffix > 9999 시 해당 도구를 skip + 경고 로그.
                for (mut key, value) in tool_name_map {
                    let original_key = key.clone();
                    let mut skipped = false;
                    if self.state.runtime.mcp_tool_name_map.contains_key(&key) {
                        // 전역 충돌 발생: suffix 부여
                        let base = key.clone();
                        let mut suffix = 2u32;
                        let mut resolved = false;
                        loop {
                            let candidate = format!("{}_{}", base, suffix);
                            if candidate.len() > App::MAX_TOOL_NAME_LEN {
                                let overflow = candidate.len() - App::MAX_TOOL_NAME_LEN;
                                let trimmed_base = &base[..base.len().saturating_sub(overflow)];
                                let trimmed_candidate = format!("{}_{}", trimmed_base, suffix);
                                if !self
                                    .state
                                    .runtime
                                    .mcp_tool_name_map
                                    .contains_key(&trimmed_candidate)
                                {
                                    key = trimmed_candidate;
                                    resolved = true;
                                    break;
                                }
                            } else if !self
                                .state
                                .runtime
                                .mcp_tool_name_map
                                .contains_key(&candidate)
                            {
                                key = candidate;
                                resolved = true;
                                break;
                            }
                            suffix += 1;
                            if suffix > 9999 {
                                break;
                            }
                        }
                        if !resolved {
                            // [v3.3.8] 감사 MEDIUM-1: skip 시 schemas에서도 해당 항목 제거.
                            // 이전: map insert만 건너뛰고 schema는 그대로 cache에 push됨.
                            // 수정: schemas에서 original_key를 가진 항목을 제거하여
                            // LLM에 노출되지만 라우팅 불가능한 도구가 생기지 않도록 방지.
                            schemas.retain(|s| {
                                s.get("function")
                                    .and_then(|f| f.get("name"))
                                    .and_then(|n| n.as_str())
                                    != Some(original_key.as_str())
                            });
                            let warn_msg = format!(
                                "[MCP] 경고: 도구 '{}' 전역 충돌 해소 실패 (suffix 한계 초과). 건너뜁니다.",
                                original_key
                            );
                            self.state.runtime.logs_buffer.push(warn_msg.clone());
                            // [v3.3.8] 감사 LOW-1: 타임라인 Notice로도 표시하여 UX 일관성 확보.
                            let mut block = crate::app::state::TimelineBlock::new(
                                crate::app::state::TimelineBlockKind::Notice,
                                format!("MCP 도구 '{}' 충돌 건너뜀", original_key),
                            );
                            block.status = crate::app::state::BlockStatus::Error;
                            block
                                .body
                                .push(crate::app::state::BlockSection::Markdown(warn_msg));
                            self.state.ui.timeline.push(block);
                            skipped = true;
                        }
                    }
                    if skipped {
                        continue;
                    }
                    // [v3.3.7] key가 변경되었으면 대응하는 schema의 function.name도 동기화.
                    // LLM에 노출되는 tool schema name과 mcp_tool_name_map key가 일치해야
                    // LLM이 호출한 도구명으로 라우팅이 가능함.
                    if key != original_key {
                        for schema in &mut schemas {
                            if let Some(func) = schema.get_mut("function").filter(|f| {
                                f.get("name").and_then(|n| n.as_str())
                                    == Some(original_key.as_str())
                            }) {
                                func["name"] = serde_json::Value::String(key.clone());
                                break;
                            }
                        }
                    }
                    self.state.runtime.mcp_tool_name_map.insert(key, value);
                }
                // [v3.3.7] 충돌 해소가 완료된 schemas를 cache에 push
                // [v3.3.8] skip된 도구는 이미 schemas에서 retain으로 제거됨
                for schema in schemas {
                    self.state.runtime.mcp_tools_cache.push(schema);
                }
                self.state
                    .runtime
                    .logs_buffer
                    .push(format!("[MCP] 서버 '{}' 로드 완료", name));
            }
            // [v3.3.1] 감사 MEDIUM-1 수정: MCP 서버 로드 실패 시 사용자에게 피드백 제공.
            // 기존에는 spawn/list_tools 실패가 완전히 침묵 처리되어
            // 사용자는 "도구가 안 보임"만 경험했음. 이제 타임라인 + 로그에 에러를 표시.
            action::Action::McpLoadFailed(name, error) => {
                self.state
                    .runtime
                    .logs_buffer
                    .push(format!("[MCP] 서버 '{}' 로드 실패: {}", name, error));
                let mut block = crate::app::state::TimelineBlock::new(
                    crate::app::state::TimelineBlockKind::Notice,
                    format!("MCP 서버 '{}' 로드 실패", name),
                );
                block.status = crate::app::state::BlockStatus::Error;
                block
                    .body
                    .push(crate::app::state::BlockSection::Markdown(format!(
                        "MCP 서버 '{}' 연결에 실패했습니다.\n사유: {}\n\n설정을 확인하세요: `/mcp list`",
                        name, error
                    )));
                self.state.ui.timeline.push(block);
            }

            // ======================================================================
            // [v3.7.0] Phase 47 Task Q-3: Interactive Planning Questionnaire
            // ======================================================================
            action::Action::ShowQuestionnaire(questions, tool_call_id, tool_index) => {
                // QuestionnaireState 생성 및 UI 모달 활성화
                self.state.ui.questionnaire =
                    Some(crate::domain::questionnaire::QuestionnaireState::new(
                        questions.clone(),
                        tool_call_id,
                        tool_index,
                    ));

                // 타임라인에 알림 블록 추가
                let question_count = questions.len();
                let mut block = crate::app::state::TimelineBlock::new(
                    crate::app::state::TimelineBlockKind::Approval,
                    format!("📋 명확화 질문 ({}건)", question_count),
                );
                block.status = crate::app::state::BlockStatus::NeedsApproval;
                block.body.push(crate::app::state::BlockSection::Markdown(
                    "AI가 요구사항을 명확히 하기 위해 질문을 보냈습니다.\n화살표 키(↑↓)로 선택하고 Enter로 답변하세요.".to_string(),
                ));
                self.state.ui.timeline.push(block);
                self.state.ui.timeline_follow_tail = true;
            }

            action::Action::QuestionnaireCompleted => {
                // QuestionnaireState에서 답변을 수집하여 ToolResult로 조립
                if let Some(qs) = self.state.ui.questionnaire.take() {
                    let result = qs.build_result();
                    let result_json =
                        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string());

                    let tool_result = crate::domain::tool_result::ToolResult {
                        tool_name: "AskClarification".to_string(),
                        tool_call_id: qs.tool_call_id,
                        stdout: result_json,
                        stderr: String::new(),
                        exit_code: 0,
                        is_error: false,
                        is_truncated: false,
                        original_size_bytes: None,
                        affected_paths: Vec::new(),
                    };

                    // ToolFinished Action으로 이벤트 루프에 태워 LLM에 피드백 전송
                    self.handle_action(action::Action::ToolFinished(
                        Box::new(tool_result),
                        qs.tool_index,
                    ));

                    // 타임라인의 Approval 블록 상태를 Done으로 갱신
                    for block in self.state.ui.timeline.iter_mut().rev() {
                        if block.kind == crate::app::state::TimelineBlockKind::Approval
                            && block.status == crate::app::state::BlockStatus::NeedsApproval
                            && block.title.contains("명확화")
                        {
                            block.status = crate::app::state::BlockStatus::Done;
                            // 답변 요약을 블록에 추가
                            let mut summary = "**답변 요약:**\n".to_string();
                            for (id, answer) in &result.answers {
                                summary.push_str(&format!("- {}: {}\n", id, answer));
                            }
                            block
                                .body
                                .push(crate::app::state::BlockSection::Markdown(summary));
                            break;
                        }
                    }
                }
            }
        }
    }

    /// [v2.5.0] Phase 33: Ordered Aggregation for parallel tool executions
    pub(crate) fn flush_pending_tool_outcomes(&mut self) {
        let mut outcomes = std::mem::take(&mut self.state.runtime.pending_tool_outcomes);
        outcomes.sort_by_key(|k| k.0);

        for (_, outcome) in outcomes {
            match outcome {
                crate::app::state::ToolOutcome::Success(res) => {
                    // 원문은 logs_buffer에 보존
                    self.state.runtime.logs_buffer.push(format!(
                        "[{}] exit={} stdout={} stderr={}",
                        res.tool_name,
                        res.exit_code,
                        res.stdout.chars().take(500).collect::<String>(),
                        res.stderr.chars().take(500).collect::<String>(),
                    ));

                    let mut content = format!(
                        "[Tool Result] {}\nExit Code: {}\nSTDOUT: {}\nSTDERR: {}",
                        res.tool_name, res.exit_code, res.stdout, res.stderr
                    );

                    if res.is_truncated {
                        let size_info = res
                            .original_size_bytes
                            .map(|s| format!("{}", s))
                            .unwrap_or_else(|| "Unknown".to_string());
                        let metadata = format!(
                            "\n\n[SYSTEM: 이 결과는 너무 길어서 일부가 절단되었습니다. 원래 크기: {} bytes]",
                            size_info
                        );
                        content.push_str(&metadata);
                    }

                    self.state
                        .domain
                        .session
                        .add_message(crate::providers::types::ChatMessage {
                            role: crate::providers::types::Role::Tool,
                            content: Some(content),
                            tool_calls: None,
                            tool_call_id: res.tool_call_id.clone(),
                            pinned: false,
                        });
                }
                crate::app::state::ToolOutcome::Error(e, tool_call_id) => {
                    let mut failure_detail = e.to_actionable().to_string();
                    self.mask_secrets(&mut failure_detail);
                    self.state
                        .domain
                        .session
                        .add_message(crate::providers::types::ChatMessage {
                            role: crate::providers::types::Role::Tool,
                            content: Some(format!("[Tool Execution Failed] {}", failure_detail)),
                            tool_calls: None,
                            tool_call_id,
                            pinned: false,
                        });
                }
            }
        }
    }

    // [v1.8.0] Phase 26: 슬라이딩 윈도우 기반 스트리밍 API 키 마스킹
    fn mask_secrets(&mut self, text: &mut String) {
        if self.state.runtime.secret_mask_regex.is_none()
            && let Some(settings) = &self.state.domain.settings
        {
            let mut keys = Vec::new();
            let mut max_len = 0;
            for k in settings.encrypted_keys.keys() {
                if let Ok(val) = crate::infra::secret_store::get_api_key(settings, k) {
                    let val_str = secrecy::ExposeSecret::expose_secret(&val);
                    if val_str.len() > 5 {
                        keys.push(regex::escape(val_str));
                        max_len = max_len.max(val_str.len());
                    }
                }
            }
            if !keys.is_empty() {
                let pattern = format!("(?:{})", keys.join("|"));
                if let Ok(re) = regex::Regex::new(&pattern) {
                    self.state.runtime.secret_mask_regex = Some(re);
                    self.state.runtime.streaming_masker.max_match_len = max_len;
                }
            } else {
                // 빈 정규식을 피하기 위해 특수한 매치되지 않는 정규식 넣기
                self.state.runtime.secret_mask_regex = regex::Regex::new(r"a^").ok();
            }
        }

        if let Some(re) = &self.state.runtime.secret_mask_regex {
            let max_match_len = self.state.runtime.streaming_masker.max_match_len;
            if max_match_len > 0 {
                let mut window =
                    std::mem::take(&mut self.state.runtime.streaming_masker.trailing_buffer);
                window.push_str(text);

                let masked_window = re.replace_all(&window, "[REDACTED]").to_string();

                // 만약 마스킹 처리된 내용이 있으면, text를 갱신
                // 주의: REDACTED 처리로 인해 문자열 길이가 달라졌을 수 있음
                if masked_window != window {
                    *text = masked_window;
                }

                // 새로운 trailing buffer 저장 (최대 max_match_len 바이트)
                let len = text.len();
                let trailing_len = max_match_len.min(len);
                if trailing_len > 0 {
                    // 유효한 UTF-8 경계를 찾아서 저장
                    let mut start_idx = len - trailing_len;
                    while !text.is_char_boundary(start_idx) && start_idx > 0 {
                        start_idx -= 1;
                    }
                    self.state.runtime.streaming_masker.trailing_buffer =
                        text[start_idx..].to_string();
                }
            } else if re.is_match(text) {
                let masked = re.replace_all(text, "[REDACTED]").to_string();
                *text = masked;
            }
        }
    }

    /// [v0.1.0-beta.18] ToolResult에서 2~4줄 요약을 생성.
    /// 타임라인에는 이 요약만 표시하고, 원문은 Logs 탭에 보존.
    fn generate_tool_summary(res: &crate::domain::tool_result::ToolResult) -> String {
        let status_icon = if res.is_error { "❌" } else { "✅" };
        let mut summary = format!("{} {} (exit {})", status_icon, res.tool_name, res.exit_code);

        // stdout이 있으면 전체(또는 특정 도구만 전체) 표시
        if !res.stdout.is_empty() {
            if res.tool_name == "ReplaceFileContent" || res.tool_name == "WriteFile" {
                summary.push_str(&format!("\n{}", res.stdout));
            } else {
                let first_lines: String = res.stdout.lines().take(2).collect::<Vec<_>>().join("\n");
                if !first_lines.is_empty() {
                    summary.push_str(&format!("\n   {}", first_lines));
                }
            }
        }
        // stderr가 있으면 첫 1줄만 표시
        if !res.stderr.is_empty()
            && let Some(first) = res.stderr.lines().next()
        {
            summary.push_str(&format!("\n   ⚠ {}", first));
        }
        summary
    }

    /// [v0.1.0-beta.10] ModelsFetched 이벤트 처리: FetchSource에 따라 정확한 상태 슬롯으로 라우팅.
    /// 이전에는 config.is_open으로 판별하여, 팝업을 닫으면 결과가 wizard로 잘못 흐르는 결함이 있었음.
    /// [v0.1.0-beta.21] 에러 타입을 String → ProviderError로 구조화.
    fn handle_models_fetched(
        &mut self,
        res: Result<Vec<String>, crate::domain::error::ProviderError>,
        source: action::FetchSource,
    ) {
        match source {
            action::FetchSource::Config => {
                self.state.ui.config.is_loading = false;
                match res {
                    Ok(models) => {
                        self.state.ui.config.available_models = models;
                        self.state.ui.config.cursor_index = 0;
                        self.state.ui.config.err_msg = None;
                    }
                    Err(e) => {
                        self.state.ui.config.err_msg = Some(e.to_string());
                        // [v0.1.0-beta.10] 6차 감사 H-1: 검증 실패 시 in-memory 설정 롤백
                        if let (Some(old_p), Some(old_m)) = (
                            self.state.ui.config.rollback_provider.take(),
                            self.state.ui.config.rollback_model.take(),
                        ) && let Some(s) = &mut self.state.domain.settings
                        {
                            s.default_provider = old_p;
                            s.default_model = old_m;
                        }
                        self.state.ui.config.active_popup = state::ConfigPopup::Dashboard;
                    }
                }
            }
            action::FetchSource::Wizard => {
                self.state.ui.wizard.is_loading_models = false;
                match res {
                    Ok(models) => {
                        self.state.ui.wizard.available_models = models;
                        self.state.ui.wizard.cursor_index = 0;
                        self.state.ui.wizard.err_msg = None;
                    }
                    Err(e) => {
                        self.state.ui.wizard.err_msg = Some(e.to_string());
                    }
                }
            }
        }
    }

    /// CredentialValidated 이벤트 처리: 검증 성공 시 모델 목록 조회 진행, 실패 시 에러 표시.
    /// [v0.1.0-beta.21] 에러 타입을 String → ProviderError로 구조화.
    fn handle_credential_validated(
        &mut self,
        res: Result<(), crate::domain::error::ProviderError>,
    ) {
        match res {
            Ok(()) => {
                // 검증 성공: 이제 fetch_models 진행
                self.state.ui.wizard.step = state::WizardStep::ModelSelection;
                self.state.ui.wizard.is_loading_models = true;
                self.state.ui.wizard.cursor_index = 0;

                let tx = self.action_tx.clone();
                let provider = self
                    .state
                    .ui
                    .wizard
                    .selected_provider
                    .clone()
                    .unwrap_or(crate::domain::provider::ProviderKind::OpenRouter);
                let api_key = self.state.ui.wizard.api_key_input.clone();

                tokio::spawn(async move {
                    let adapter = crate::providers::registry::get_adapter(&provider);
                    match adapter.fetch_models(&api_key).await {
                        Ok(models) => {
                            let _ = tx
                                .send(event_loop::Event::Action(action::Action::ModelsFetched(
                                    Ok(models),
                                    action::FetchSource::Wizard,
                                )))
                                .await;
                        }
                        Err(e) => {
                            // [v0.1.0-beta.21] ProviderError 구조화
                            let _ = tx
                                .send(event_loop::Event::Action(action::Action::ModelsFetched(
                                    Err(crate::domain::error::ProviderError::NetworkFailure(
                                        e.to_string(),
                                    )),
                                    action::FetchSource::Wizard,
                                )))
                                .await;
                        }
                    }
                });
            }
            Err(e) => {
                // 검증 실패: ApiKeyInput 단계로 복귀하고 에러 표시
                self.state.ui.wizard.is_loading_models = false;
                self.state.ui.wizard.step = state::WizardStep::ApiKeyInput;
                self.state.ui.wizard.err_msg = Some(format!("API 키 검증 실패: {}", e));
                // [v1.0.0] State 누수 방지: 인증 실패 시 입력 버퍼 초기화 (ClearBuffer)
                self.state.ui.wizard.api_key_input.clear();
            }
        }
    }

    /// 키보드 입력 이벤트 처리: 전역 단축키, 승인 UI, 위자드, Fuzzy Finder, Composer를 순차 라우팅.
    pub(crate) fn handle_input(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};

        // [v0.1.0-beta.26] Trust Gate 가 열려있으면 전역 단축키를 완전히 차단하고, 상하 방향키와 엔터/Ctrl-C 만 허용한다.
        if let crate::app::state::TrustGatePopup::Open { .. } = self.state.ui.trust_gate.popup {
            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.state.should_quit = true;
                }
                KeyCode::Up => {
                    if self.state.ui.trust_gate.cursor_index > 0 {
                        self.state.ui.trust_gate.cursor_index -= 1;
                    }
                }
                KeyCode::Down => {
                    if self.state.ui.trust_gate.cursor_index < 2 {
                        self.state.ui.trust_gate.cursor_index += 1;
                    }
                }
                KeyCode::Enter => {
                    self.handle_enter_key();
                }
                _ => {} // 다른 모든 입력 무시
            }
            return; // [v2.5.0] Trust Gate 활성 시 나머지 키 핸들러로의 폴스루 차단
        }

        // [v2.4.0] Phase 32: Help Overlay 닫기 및 토글 로직
        if self.state.ui.show_help_overlay {
            if key.code == KeyCode::Esc
                || key.code == KeyCode::Enter
                || key.code == KeyCode::Char('?')
            {
                self.state.ui.show_help_overlay = false;
            }
            return;
        }

        // [v3.7.0] Phase 47 Task Q-2: Questionnaire 모달 활성 시 키 입력 인터셉트.
        // 다른 모든 키 핸들러보다 우선하여 질문 폼의 탐색/선택/입력을 처리.
        if self.state.ui.questionnaire.is_some() {
            self.handle_questionnaire_key(key);
            return;
        }

        if key.code == KeyCode::F(1)
            || (key.code == KeyCode::Char('?')
                && self.state.ui.focused_pane != crate::app::state::FocusedPane::Composer)
        {
            self.state.ui.show_help_overlay = true;
            return;
        }

        match key.code {
            // 전역 단축키: Ctrl+C 종료 또는 실행 취소
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if !self.state.runtime.active_tool_cancel_tokens.is_empty() {
                    for token in self.state.runtime.active_tool_cancel_tokens.values() {
                        token.cancel();
                    }
                    self.state.runtime.active_tool_cancel_tokens.clear();
                    self.state
                        .runtime
                        .logs_buffer
                        .push("[App] 사용자 취소 요청 (Ctrl+C): 모든 도구 실행 중단".to_string());
                } else {
                    self.state.should_quit = true;
                }
            }

            KeyCode::F(2) => {
                self.state.ui.show_inspector = !self.state.ui.show_inspector;
                if self.state.ui.show_inspector {
                    self.state.ui.focused_pane = crate::app::state::FocusedPane::Inspector;
                } else if self.state.ui.focused_pane == crate::app::state::FocusedPane::Inspector {
                    self.state.ui.focused_pane = crate::app::state::FocusedPane::Composer;
                }
            }
            KeyCode::Char('k') | KeyCode::Char('K')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.state.ui.palette.is_open = !self.state.ui.palette.is_open;
                if self.state.ui.palette.is_open {
                    self.state.ui.focused_pane = crate::app::state::FocusedPane::Palette;
                    self.state.ui.palette.query.clear();
                    self.state.ui.palette.cursor = 0;
                    self.update_palette_matches();
                } else if self.state.ui.focused_pane == crate::app::state::FocusedPane::Palette {
                    self.state.ui.focused_pane = crate::app::state::FocusedPane::Composer;
                }
            }
            KeyCode::Esc => {
                if self.state.ui.palette.is_open {
                    self.state.ui.palette.is_open = false;
                    if self.state.ui.focused_pane == crate::app::state::FocusedPane::Palette {
                        self.state.ui.focused_pane = crate::app::state::FocusedPane::Composer;
                    }
                } else if self.state.ui.show_inspector
                    && self.state.ui.focused_pane == crate::app::state::FocusedPane::Inspector
                {
                    self.state.ui.show_inspector = false;
                    self.state.ui.focused_pane = crate::app::state::FocusedPane::Composer;
                } else if self.state.ui.slash_menu.is_open {
                    // [v0.1.0-beta.16] 슬래시 메뉴 닫기
                    self.state.ui.slash_menu.is_open = false;
                    self.state.ui.slash_menu.filter.clear();
                } else if self.state.ui.fuzzy.is_open {
                    self.state.ui.fuzzy.is_open = false;
                } else if self.state.ui.config.is_open {
                    if self.state.ui.config.active_popup != state::ConfigPopup::Dashboard {
                        // [v0.1.0-beta.11] 7차 감사 H-2: 사용자 취소 시 롤백.
                        // ProviderList→ModelList 진행 중 Esc로 돌아오면,
                        // in-memory settings를 이전 provider/model로 복구해야 함.
                        if let (Some(old_p), Some(old_m)) = (
                            self.state.ui.config.rollback_provider.take(),
                            self.state.ui.config.rollback_model.take(),
                        ) && let Some(s) = &mut self.state.domain.settings
                        {
                            s.default_provider = old_p;
                            s.default_model = old_m;
                        }
                        self.state.ui.config.err_msg = None;
                        self.state.ui.config.active_popup = state::ConfigPopup::Dashboard;
                    } else {
                        self.state.ui.config.is_open = false;
                        self.state.ui.config.err_msg = None;
                    }
                } else if self.state.ui.is_wizard_open && self.state.ui.wizard.err_msg.is_some() {
                    // [v0.1.0-beta.8] M-1: Error state에서 Esc 시 Wizard 홈으로 회귀
                    self.state.ui.wizard.step = state::WizardStep::ProviderSelection;
                    self.state.ui.wizard.err_msg = None;
                    self.state.ui.wizard.api_key_input.clear();
                    self.state.ui.wizard.cursor_index = 0;
                } else if !self.state.runtime.active_tool_cancel_tokens.is_empty() {
                    for token in self.state.runtime.active_tool_cancel_tokens.values() {
                        token.cancel();
                    }
                    self.state.runtime.active_tool_cancel_tokens.clear();
                    self.state
                        .runtime
                        .logs_buffer
                        .push("[App] 사용자 취소 요청 (ESC): 모든 도구 실행 중단".to_string());
                } else {
                    self.state.should_quit = true;
                }
            }
            KeyCode::Char(c) => {
                self.handle_char_input(c);
            }
            KeyCode::Up => {
                self.handle_up_key();
            }
            KeyCode::Down => {
                self.handle_down_key();
            }
            KeyCode::Backspace => {
                self.handle_backspace();
            }
            KeyCode::Enter => {
                if key.modifiers.contains(KeyModifiers::SHIFT)
                    && !self.state.ui.is_wizard_open
                    && !self.state.ui.fuzzy.is_open
                    && !self.state.ui.slash_menu.is_open
                    && !self.state.ui.config.is_open
                    && !self.state.ui.palette.is_open
                {
                    if self.state.ui.focused_pane == crate::app::state::FocusedPane::Composer {
                        self.state.ui.composer.input_buffer.push('\n');
                    }
                } else {
                    self.handle_enter_key();
                }
            }
            KeyCode::Tab | KeyCode::BackTab => {
                if self.state.ui.is_wizard_open {
                    // [v1.0.0] 위저드 탭 포커스 순환 강제 (순서 꼬임 해결)
                    // Provider -> ApiKey -> Model -> SaveButton
                    let reverse =
                        key.code == KeyCode::BackTab || key.modifiers.contains(KeyModifiers::SHIFT);
                    self.state.ui.wizard.step = match self.state.ui.wizard.step {
                        state::WizardStep::ProviderSelection => {
                            if reverse {
                                state::WizardStep::Saving
                            } else {
                                state::WizardStep::ApiKeyInput
                            }
                        }
                        state::WizardStep::ApiKeyInput => {
                            if reverse {
                                state::WizardStep::ProviderSelection
                            } else {
                                state::WizardStep::ModelSelection
                            }
                        }
                        state::WizardStep::ModelSelection => {
                            if reverse {
                                state::WizardStep::ApiKeyInput
                            } else {
                                state::WizardStep::Saving
                            }
                        }
                        state::WizardStep::Saving => {
                            if reverse {
                                state::WizardStep::ModelSelection
                            } else {
                                state::WizardStep::ProviderSelection
                            }
                        }
                    };
                } else {
                    use crate::domain::session::AppMode;
                    self.state.domain.session.mode = match self.state.domain.session.mode {
                        AppMode::Plan => AppMode::Run,
                        AppMode::Run => AppMode::Plan,
                    };
                }
            }
            // [v0.1.0-beta.22] 타임라인 스크롤: PageUp/PageDown으로 긴 응답을 탐색.
            // 위자드, Fuzzy, 설정 팝업이 열려 있을 때는 동작하지 않음.
            KeyCode::PageUp => {
                if !self.state.ui.is_wizard_open
                    && !self.state.ui.fuzzy.is_open
                    && !self.state.ui.config.is_open
                {
                    match self.state.ui.focused_pane {
                        crate::app::state::FocusedPane::Inspector => {
                            self.state
                                .ui
                                .inspector_scroll
                                .set(self.state.ui.inspector_scroll.get().saturating_add(5));
                        }
                        _ => self.scroll_timeline_up(5),
                    }
                }
            }
            KeyCode::PageDown => {
                if !self.state.ui.is_wizard_open
                    && !self.state.ui.fuzzy.is_open
                    && !self.state.ui.config.is_open
                {
                    match self.state.ui.focused_pane {
                        crate::app::state::FocusedPane::Inspector => {
                            self.state
                                .ui
                                .inspector_scroll
                                .set(self.state.ui.inspector_scroll.get().saturating_sub(5));
                        }
                        _ => self.scroll_timeline_down(5),
                    }
                }
            }
            _ => {}
        }
    }

    /// 문자 입력 처리: 승인 대기 → 위자드 → Slash Menu → Fuzzy Finder → Composer 순으로 라우팅.
    fn handle_char_input(&mut self, c: char) {
        if let crate::app::state::TrustGatePopup::Open { .. } = self.state.ui.trust_gate.popup {
            return;
        }
        if self.state.runtime.approval.pending_tool.is_some() {
            if c == 'y' {
                self.handle_tool_approval(true);
            } else if c == 'n' {
                self.handle_tool_approval(false);
            }
        } else if self.state.ui.is_wizard_open {
            if self.state.ui.wizard.step == state::WizardStep::ApiKeyInput {
                // [v1.0.0] 에러 잔류 방지: 첫 입력 시 에러 메시지 초기화
                self.state.ui.wizard.err_msg = None;
                self.state.ui.wizard.api_key_input.push(c);
            }
        } else if self.state.ui.palette.is_open {
            self.state.ui.palette.query.push(c);
            self.update_palette_matches();
        } else if self.state.ui.slash_menu.is_open {
            // [v0.1.0-beta.16] 슬래시 메뉴 활성 상태: 필터 문자 추가
            self.state.ui.slash_menu.filter.push(c);
            self.state.ui.slash_menu.update_matches();
        } else if self.state.ui.fuzzy.is_open {
            self.state.ui.fuzzy.input.push(c);
            self.update_fuzzy_matches();
        } else {
            if c == '/' && self.state.ui.composer.input_buffer.is_empty() {
                // [v0.1.0-beta.16] 빈 Composer에서 / 입력 시 슬래시 메뉴 활성화
                self.state.ui.slash_menu.is_open = true;
                self.state.ui.slash_menu.filter.clear();
                self.state.ui.slash_menu.cursor = 0;
                self.state.ui.slash_menu.update_matches();
            } else if c == '@' {
                self.state.ui.fuzzy.is_open = true;
                self.state.ui.fuzzy.mode = crate::app::state::FuzzyMode::Files;
                self.state.ui.fuzzy.input.clear();
                self.state.ui.fuzzy.matches.clear();
                self.state.ui.fuzzy.cursor = 0;
                self.update_fuzzy_matches();
            } else if c == '!' && self.state.ui.composer.input_buffer.is_empty() {
                self.state.ui.composer.input_buffer.push(c); // ! 유지
                self.state.ui.fuzzy.is_open = true;
                self.state.ui.fuzzy.mode = crate::app::state::FuzzyMode::Macros;
                self.state.ui.fuzzy.input.clear();
                self.state.ui.fuzzy.matches.clear();
                self.state.ui.fuzzy.cursor = 0;
                self.update_fuzzy_matches();
            } else if c == 'y'
                && self.state.ui.focused_pane != crate::app::state::FocusedPane::Composer
            {
                // [v2.2.0] Phase 30: TUI Clipboard Integration
                self.copy_focused_content_to_clipboard();
            } else {
                if self.state.ui.focused_pane != crate::app::state::FocusedPane::Composer {
                    self.state.ui.focused_pane = crate::app::state::FocusedPane::Composer;
                }
                self.state.ui.composer.input_buffer.push(c);
            }
        }
    }

    /// [v2.2.0] Phase 30: arboard 연동 클립보드 복사
    fn copy_focused_content_to_clipboard(&mut self) {
        let content = match self.state.ui.focused_pane {
            crate::app::state::FocusedPane::Inspector => self.state.runtime.logs_buffer.join("\n"),
            crate::app::state::FocusedPane::Timeline => {
                // 타임라인 포커싱 시 최근 어시스턴트 메시지를 복사
                let msgs = &self.state.domain.session.messages;
                if let Some(msg) = msgs
                    .iter()
                    .rev()
                    .find(|m| m.role == crate::providers::types::Role::Assistant)
                {
                    msg.content.clone().unwrap_or_default()
                } else {
                    String::new()
                }
            }
            _ => String::new(),
        };

        if !content.is_empty() {
            let now = std::time::Instant::now();
            let expires_at = now + std::time::Duration::from_secs(2);

            match arboard::Clipboard::new().and_then(|mut c| c.set_text(&content)) {
                Ok(_) => {
                    self.state
                        .runtime
                        .logs_buffer
                        .push(format!("[Clipboard] {} bytes 복사 성공", content.len()));
                    self.state.ui.toast = Some(crate::app::state::ToastNotification {
                        message: "복사 완료!".into(),
                        expires_at,
                        is_error: false,
                    });
                }
                Err(e) => {
                    self.state
                        .runtime
                        .logs_buffer
                        .push(format!("[Clipboard] 복사 실패: {}", e));
                    self.state.ui.toast = Some(crate::app::state::ToastNotification {
                        message: "복사 실패!".into(),
                        expires_at,
                        is_error: true,
                    });
                }
            }
        }
    }

    /// Up 화살표 키 처리: Slash Menu, Fuzzy Finder, Config, Wizard 각 모드별 커서 이동.
    fn handle_up_key(&mut self) {
        if let crate::app::state::TrustGatePopup::Open { .. } = self.state.ui.trust_gate.popup {
            if self.state.ui.trust_gate.cursor_index > 0 {
                self.state.ui.trust_gate.cursor_index -= 1;
            }
            return;
        }
        if self.state.ui.palette.is_open {
            if self.state.ui.palette.cursor > 0 {
                self.state.ui.palette.cursor -= 1;
            }
        } else if self.state.ui.slash_menu.is_open {
            if self.state.ui.slash_menu.cursor > 0 {
                self.state.ui.slash_menu.cursor -= 1;
            }
        } else if self.state.ui.fuzzy.is_open {
            if self.state.ui.fuzzy.cursor > 0 {
                self.state.ui.fuzzy.cursor -= 1;
            }
        } else if self.state.ui.config.is_open && self.state.ui.config.cursor_index > 0 {
            self.state.ui.config.cursor_index -= 1;
        } else if self.state.ui.is_wizard_open && self.state.ui.wizard.cursor_index > 0 {
            self.state.ui.wizard.cursor_index -= 1;
        } else if self.state.ui.focused_pane == crate::app::state::FocusedPane::Timeline {
            if self.state.ui.timeline_cursor > 0 {
                self.state.ui.timeline_cursor -= 1;
            }
        } else if self.state.ui.focused_pane == crate::app::state::FocusedPane::Inspector {
            self.state
                .ui
                .inspector_scroll
                .set(self.state.ui.inspector_scroll.get().saturating_add(1));
        } else if !self.state.ui.composer.history.is_empty()
            && self.state.ui.focused_pane == crate::app::state::FocusedPane::Composer
        {
            // Composer History (Up)
            let len = self.state.ui.composer.history.len();
            let new_idx = match self.state.ui.composer.history_idx {
                Some(idx) => idx.saturating_sub(1),
                None => len.saturating_sub(1),
            };
            self.state.ui.composer.history_idx = Some(new_idx);
            self.state.ui.composer.input_buffer = self.state.ui.composer.history[new_idx].clone();
        }
    }

    /// Down 화살표 키 처리: 각 모드별 리스트의 최대값까지 커서 이동.
    fn handle_down_key(&mut self) {
        if let crate::app::state::TrustGatePopup::Open { .. } = self.state.ui.trust_gate.popup {
            if self.state.ui.trust_gate.cursor_index < 2 {
                self.state.ui.trust_gate.cursor_index += 1;
            }
            return;
        }
        if self.state.ui.palette.is_open {
            if self.state.ui.palette.cursor + 1 < self.state.ui.palette.results.len() {
                self.state.ui.palette.cursor += 1;
            }
        } else if self.state.ui.slash_menu.is_open {
            if self.state.ui.slash_menu.cursor + 1 < self.state.ui.slash_menu.matches.len() {
                self.state.ui.slash_menu.cursor += 1;
            }
        } else if self.state.ui.fuzzy.is_open {
            if self.state.ui.fuzzy.cursor + 1 < self.state.ui.fuzzy.matches.len().min(3) {
                self.state.ui.fuzzy.cursor += 1;
            }
        } else if self.state.ui.config.is_open {
            let max = match self.state.ui.config.active_popup {
                state::ConfigPopup::Dashboard => 4,
                state::ConfigPopup::ProviderList => {
                    4 + self
                        .state
                        .domain
                        .settings
                        .as_ref()
                        .map(|s| s.custom_providers.len())
                        .unwrap_or(0)
                }
                state::ConfigPopup::ModelList => self
                    .state
                    .ui
                    .config
                    .available_models
                    .len()
                    .saturating_sub(1),
            };
            if self.state.ui.config.cursor_index < max {
                self.state.ui.config.cursor_index += 1;
            }
        } else if self.state.ui.is_wizard_open {
            let max = match self.state.ui.wizard.step {
                state::WizardStep::ProviderSelection => 4,
                state::WizardStep::ModelSelection => self
                    .state
                    .ui
                    .wizard
                    .available_models
                    .len()
                    .saturating_sub(1),
                _ => 0,
            };
            if self.state.ui.wizard.cursor_index < max {
                self.state.ui.wizard.cursor_index += 1;
            }
        } else if self.state.ui.focused_pane == crate::app::state::FocusedPane::Timeline {
            if self.state.ui.timeline_cursor + 1 < self.state.ui.timeline.len() {
                self.state.ui.timeline_cursor += 1;
            }
        } else if self.state.ui.focused_pane == crate::app::state::FocusedPane::Inspector {
            self.state
                .ui
                .inspector_scroll
                .set(self.state.ui.inspector_scroll.get().saturating_sub(1));
        } else if let Some(idx) = self.state.ui.composer.history_idx
            && self.state.ui.focused_pane == crate::app::state::FocusedPane::Composer
        {
            // Composer History (Down)
            let len = self.state.ui.composer.history.len();
            if idx + 1 < len {
                self.state.ui.composer.history_idx = Some(idx + 1);
                self.state.ui.composer.input_buffer =
                    self.state.ui.composer.history[idx + 1].clone();
            } else {
                self.state.ui.composer.history_idx = None;
                self.state.ui.composer.input_buffer.clear();
            }
        }
    }

    /// Backspace 키 처리: Slash Menu, Fuzzy Finder, Wizard API 키, Composer 각각의 버퍼 삭제.
    fn handle_backspace(&mut self) {
        if self.state.ui.palette.is_open {
            if self.state.ui.palette.query.is_empty() {
                self.state.ui.palette.is_open = false;
                self.state.ui.focused_pane = crate::app::state::FocusedPane::Composer;
            } else {
                self.state.ui.palette.query.pop();
                self.update_palette_matches();
            }
        } else if self.state.ui.slash_menu.is_open {
            // [v0.1.0-beta.16] 필터가 비면 메뉴 닫기, 아니면 필터 문자 삭제
            if self.state.ui.slash_menu.filter.is_empty() {
                self.state.ui.slash_menu.is_open = false;
            } else {
                self.state.ui.slash_menu.filter.pop();
                self.state.ui.slash_menu.update_matches();
            }
        } else if self.state.ui.fuzzy.is_open {
            if self.state.ui.fuzzy.input.is_empty() {
                self.state.ui.fuzzy.is_open = false;
            } else {
                self.state.ui.fuzzy.input.pop();
                self.update_fuzzy_matches();
            }
        } else if self.state.ui.is_wizard_open {
            if self.state.ui.wizard.step == state::WizardStep::ApiKeyInput {
                // [v1.0.0] 에러 잔류 방지: 첫 백스페이스 시 에러 메시지 초기화
                self.state.ui.wizard.err_msg = None;
                self.state.ui.wizard.api_key_input.pop();
            }
        } else {
            self.state.ui.composer.input_buffer.pop();
        }
    }

    /// 마우스 이벤트 처리 (Phase 14-B)
    pub fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        if let crate::app::state::TrustGatePopup::Open { .. } = self.state.ui.trust_gate.popup {
            return; // [v0.1.0-beta.26] Trust Gate 활성화 시 마우스 이벤트 차단
        }

        use crossterm::event::{MouseButton, MouseEventKind};

        let (term_cols, term_rows) = crossterm::terminal::size().unwrap_or((100, 30));
        let target = self.mouse_target(mouse, term_cols, term_rows);

        match mouse.kind {
            MouseEventKind::ScrollUp => match target {
                MousePaneTarget::Inspector => {
                    self.state
                        .ui
                        .inspector_scroll
                        .set(self.state.ui.inspector_scroll.get().saturating_add(3));
                }
                MousePaneTarget::Timeline => self.scroll_timeline_up(3),
                _ => {}
            },
            MouseEventKind::ScrollDown => match target {
                MousePaneTarget::Inspector => {
                    self.state
                        .ui
                        .inspector_scroll
                        .set(self.state.ui.inspector_scroll.get().saturating_sub(3));
                }
                MousePaneTarget::Timeline => self.scroll_timeline_down(3),
                _ => {}
            },
            MouseEventKind::Down(MouseButton::Left) => {
                // 클릭을 통한 포커스 라우팅
                match target {
                    MousePaneTarget::Inspector => {
                        self.state.ui.focused_pane = crate::app::state::FocusedPane::Inspector;
                    }
                    MousePaneTarget::Timeline => {
                        self.state.ui.focused_pane = crate::app::state::FocusedPane::Timeline;
                    }
                    MousePaneTarget::Composer => {
                        self.state.ui.focused_pane = crate::app::state::FocusedPane::Composer;
                    }
                    MousePaneTarget::Other => {}
                }
            }
            _ => {}
        }
    }

    /// Enter 키 처리: Slash Menu 선택 → Fuzzy Finder 선택 → Config 팝업 → Wizard → Composer 제출 순으로 라우팅.
    pub(crate) fn handle_enter_key(&mut self) {
        if let crate::app::state::TrustGatePopup::Open { root } =
            self.state.ui.trust_gate.popup.clone()
        {
            let trust_state = match self.state.ui.trust_gate.cursor_index {
                0 => crate::domain::settings::WorkspaceTrustState::Trusted, // Trust & Remember
                1 => crate::domain::settings::WorkspaceTrustState::Trusted, // Trust Once
                2 => crate::domain::settings::WorkspaceTrustState::Restricted, // Restricted
                _ => crate::domain::settings::WorkspaceTrustState::Unknown,
            };

            if self.state.ui.trust_gate.cursor_index == 0 {
                // Trust & Remember
                if let Some(settings) = &mut self.state.domain.settings {
                    settings.set_workspace_trust(&root, trust_state.clone(), true);
                    let settings_clone = settings.clone();
                    let tx = self.action_tx.clone();
                    tokio::spawn(async move {
                        let res = crate::infra::config_store::save_config(&settings_clone)
                            .await
                            .map_err(|e| e.to_string());
                        let _ = tx
                            .send(crate::app::event_loop::Event::Action(
                                crate::app::action::Action::ConfigSaveFinished(res),
                            ))
                            .await;
                    });
                }
            } else if self.state.ui.trust_gate.cursor_index == 2 {
                // Restricted
                if let Some(settings) = &mut self.state.domain.settings {
                    settings.set_workspace_trust(&root, trust_state.clone(), true);
                    settings.denied_roots.push(root.clone());
                    let settings_clone = settings.clone();
                    let tx = self.action_tx.clone();
                    tokio::spawn(async move {
                        let res = crate::infra::config_store::save_config(&settings_clone)
                            .await
                            .map_err(|e| e.to_string());
                        let _ = tx
                            .send(crate::app::event_loop::Event::Action(
                                crate::app::action::Action::ConfigSaveFinished(res),
                            ))
                            .await;
                    });
                }
            } else {
                // Trust Once
                if let Some(settings) = &mut self.state.domain.settings {
                    settings.set_workspace_trust(&root, trust_state.clone(), false);
                }
            }

            self.state.ui.trust_gate.popup = crate::app::state::TrustGatePopup::Closed;
            self.state.ui.focused_pane = crate::app::state::FocusedPane::Composer;
            return;
        }

        if self.state.ui.palette.is_open {
            if !self.state.ui.palette.results.is_empty() {
                let cmd = self.state.ui.palette.results[self.state.ui.palette.cursor].id;
                self.state.ui.palette.is_open = false;
                self.state.ui.focused_pane = crate::app::state::FocusedPane::Composer;
                self.state.ui.palette.query.clear();
                // 팔레트 명령어 라우팅
                if cmd.starts_with('/') {
                    self.handle_slash_command(cmd);
                } else if cmd == "toggle_inspector" {
                    self.state.ui.show_inspector = !self.state.ui.show_inspector;
                }
            } else {
                self.state.ui.palette.is_open = false;
                self.state.ui.focused_pane = crate::app::state::FocusedPane::Composer;
            }
        } else if self.state.ui.slash_menu.is_open {
            // [v0.1.0-beta.16] 슬래시 메뉴에서 명령어 선택 → 바로 실행
            if !self.state.ui.slash_menu.matches.is_empty() {
                let (cmd, _) = self.state.ui.slash_menu.matches[self.state.ui.slash_menu.cursor];
                let cmd_str = cmd.to_string();
                self.state.ui.slash_menu.is_open = false;
                self.state.ui.slash_menu.filter.clear();
                self.handle_slash_command(&cmd_str);
            } else {
                self.state.ui.slash_menu.is_open = false;
            }
        } else if self.state.ui.fuzzy.is_open {
            if !self.state.ui.fuzzy.matches.is_empty() {
                let selected = &self.state.ui.fuzzy.matches[self.state.ui.fuzzy.cursor];
                match self.state.ui.fuzzy.mode {
                    crate::app::state::FuzzyMode::Files => {
                        self.state
                            .ui
                            .composer
                            .input_buffer
                            .push_str(&format!("@{} ", selected));
                    }
                    crate::app::state::FuzzyMode::Macros => {
                        // "build      (cargo build)" -> "cargo build"
                        let cmd = selected
                            .split('(')
                            .nth(1)
                            .unwrap_or("")
                            .trim_end_matches(')');
                        self.state.ui.composer.input_buffer.clear();
                        self.state
                            .ui
                            .composer
                            .input_buffer
                            .push_str(&format!("!{}", cmd));
                    }
                }
            }
            self.state.ui.fuzzy.is_open = false;
        } else if self.state.ui.config.is_open {
            // Config 팝업 Enter 처리 (wizard_controller.rs에 위임)
            self.handle_config_enter();
        } else if self.state.ui.is_wizard_open {
            // 위자드 Enter 처리 (wizard_controller.rs에 위임)
            self.handle_wizard_enter();
        } else if self.state.ui.focused_pane == crate::app::state::FocusedPane::Timeline {
            let cursor = self.state.ui.timeline_cursor;
            if cursor < self.state.ui.timeline.len()
                && self.state.ui.timeline[cursor].kind
                    == crate::app::state::TimelineBlockKind::ToolRun
            {
                self.state.ui.timeline[cursor].toggle_collapse();
            }
        } else {
            // Composer 제출: 슬래시 커맨드, 직접 셸, 자연어 입력 분기
            let text = self.state.ui.composer.input_buffer.trim().to_string();
            if !text.is_empty() {
                // [v0.1.0-beta.26] 진행 중(is_thinking)일 때 자연어 채팅 요청 및 명령어 실행 차단 (Race condition 방지)
                if self.state.runtime.is_thinking {
                    self.state.runtime.logs_buffer.push(
                        "[Warning] 이전 요청이 진행 중입니다. 완료 후 입력해주세요.".to_string(),
                    );
                    return;
                }

                self.state.ui.composer.input_buffer.clear();
                self.state.ui.composer.history_idx = None;

                if text.starts_with('/') {
                    self.handle_slash_command(&text);
                } else if let Some(stripped) = text.strip_prefix('!') {
                    let cmd_str = stripped.trim().to_string();
                    if !cmd_str.is_empty() {
                        self.state.ui.composer.history.push(format!("!{}", cmd_str));
                    }
                    self.handle_direct_shell_execution(cmd_str);
                } else {
                    self.dispatch_chat_request(text);
                }
            }
        }
    }

    /// Palette 매칭 로직
    fn update_palette_matches(&mut self) {
        let input = self.state.ui.palette.query.to_lowercase();
        let mut results = Vec::new();

        for cmd in self.state.ui.palette.all_commands.iter() {
            if input.is_empty() {
                results.push(cmd.clone());
            } else {
                let target = format!(
                    "{} {}",
                    cmd.title.to_lowercase(),
                    cmd.category.to_string().to_lowercase()
                );
                let mut target_chars = target.chars();
                let mut is_match = true;
                for ch in input.chars() {
                    if ch.is_whitespace() {
                        continue;
                    }
                    let mut found = false;
                    for tc in target_chars.by_ref() {
                        if tc == ch {
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        is_match = false;
                        break;
                    }
                }
                if is_match {
                    results.push(cmd.clone());
                }
            }
        }

        results.truncate(50); // 최대 50개 제한
        self.state.ui.palette.results = results;
        self.state.ui.palette.cursor = 0;
    }

    /// Fuzzy Finder 매칭 로직
    fn update_fuzzy_matches(&mut self) {
        let input = self.state.ui.fuzzy.input.clone();
        let mut matches = Vec::new();

        match self.state.ui.fuzzy.mode {
            crate::app::state::FuzzyMode::Files => {
                // 1. 특수 멘션 먼저 추가
                if input.is_empty() || "workspace".contains(&input.to_lowercase()) {
                    matches.push("workspace".to_string());
                }
                if input.is_empty() || "terminal".contains(&input.to_lowercase()) {
                    matches.push("terminal".to_string());
                }

                // 2. ignore 크레이트를 활용한 재귀적 파일 탐색
                use ignore::WalkBuilder;
                let walker = WalkBuilder::new(".").hidden(true).build();

                for entry in walker.flatten() {
                    if let Some(file_type) = entry.file_type() {
                        if !file_type.is_file() {
                            continue;
                        }

                        let path_str = entry.path().to_string_lossy().into_owned();
                        let clean_path = if let Some(stripped) = path_str.strip_prefix("./") {
                            stripped.to_string()
                        } else {
                            path_str
                        };

                        if input.is_empty()
                            || clean_path.to_lowercase().contains(&input.to_lowercase())
                        {
                            matches.push(clean_path);
                            if matches.len() > 100 {
                                break;
                            }
                        }
                    }
                }
                matches.sort();
            }
            crate::app::state::FuzzyMode::Macros => {
                // ! 모드 매크로 리스트
                let macros = vec![
                    ("build", "cargo build"),
                    ("test", "cargo test"),
                    ("run", "cargo run"),
                    ("check", "cargo check"),
                    ("fmt", "cargo fmt"),
                    ("clippy", "cargo clippy"),
                ];

                for (name, cmd) in macros {
                    if input.is_empty() || name.to_lowercase().contains(&input.to_lowercase()) {
                        matches.push(format!("{:<10} ({})", name, cmd));
                    }
                }
            }
        }

        self.state.ui.fuzzy.matches = matches;
        self.state.ui.fuzzy.cursor = 0;
    }

    /// Toolbar 상태 동기화 (Phase 15-D)
    fn sync_toolbar(&mut self) {
        use crate::app::state::{InputChip, InputChipKind};
        use crate::domain::session::AppMode;

        let mut chips = Vec::new();

        // 1. Mode Chip
        let mode_label = match self.state.domain.session.mode {
            AppMode::Plan => "PLAN",
            AppMode::Run => "RUN",
        };
        chips.push(InputChip {
            kind: InputChipKind::Mode,
            label: mode_label.to_string(),
            emphasized: true,
        });

        // 2. Path Chip (CWD)
        let cwd_raw = std::env::current_dir()
            .map(|pp| pp.display().to_string())
            .unwrap_or_else(|_| "?".to_string());
        chips.push(InputChip {
            kind: InputChipKind::Path,
            label: cwd_raw,
            emphasized: false,
        });

        // 2.5 Context Chips (최대 5개)
        let mut context_count = 0;
        for token in self.state.ui.composer.input_buffer.split_whitespace() {
            if token.starts_with('@') && token.len() > 1 {
                chips.push(InputChip {
                    kind: InputChipKind::Context,
                    label: token.to_string(),
                    emphasized: false,
                });
                context_count += 1;
                if context_count >= 5 {
                    break;
                }
            }
        }

        // 3. Policy Chip
        let policy_str = if let Some(settings) = &self.state.domain.settings {
            match settings.shell_policy {
                crate::domain::permissions::ShellPolicy::Ask => "Shell: Ask",
                crate::domain::permissions::ShellPolicy::SafeOnly => "Shell: SafeOnly",
                crate::domain::permissions::ShellPolicy::Deny => "Shell: Deny",
            }
        } else {
            "Shell: Ask"
        };
        chips.push(InputChip {
            kind: InputChipKind::Policy,
            label: policy_str.to_string(),
            emphasized: false,
        });

        // 4. Hint Chip
        chips.push(InputChip {
            kind: InputChipKind::Hint,
            label: "F2 Inspector".to_string(),
            emphasized: false,
        });
        chips.push(InputChip {
            kind: InputChipKind::Hint,
            label: "Ctrl+K Palette".to_string(),
            emphasized: false,
        });

        self.state.ui.toolbar.chips = chips;
        self.state.ui.toolbar.multiline = self.state.ui.composer.input_buffer.contains('\n');
    }

    // ======================================================================
    // [v3.7.0] Phase 47 Task Q-2: Questionnaire 키 입력 핸들러.
    // Questionnaire 모달이 활성화되어 있을 때의 전체 키 입력을 관리.
    // - 객관식: ↑↓으로 옵션 탐색, Enter로 선택
    // - 주관식: 텍스트 입력 후 Enter로 제출
    // - allow_custom: 마지막 옵션("직접 입력")에서 Enter 시 입력 모드 전환
    // - Esc: Questionnaire 취소 (빈 ToolResult 반환)
    // ======================================================================
    fn handle_questionnaire_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;

        let Some(ref mut qs) = self.state.ui.questionnaire else {
            return;
        };

        match key.code {
            // Esc: Questionnaire 취소
            KeyCode::Esc => {
                let tool_call_id = qs.tool_call_id.clone();
                let tool_index = qs.tool_index;
                self.state.ui.questionnaire = None;

                // 빈 결과를 ToolResult로 반환하여 LLM에 취소를 알림
                let tool_result = crate::domain::tool_result::ToolResult {
                    tool_name: "AskClarification".to_string(),
                    tool_call_id,
                    stdout: "{\"answers\":{},\"cancelled\":true}".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                    is_error: false,
                    is_truncated: false,
                    original_size_bytes: None,
                    affected_paths: Vec::new(),
                };
                self.handle_action(action::Action::ToolFinished(
                    Box::new(tool_result),
                    tool_index,
                ));
            }

            // 위쪽 화살표: 옵션 커서 이동
            KeyCode::Up => {
                if !qs.is_custom_input_mode && qs.option_cursor > 0 {
                    qs.option_cursor -= 1;
                }
            }

            // 아래쪽 화살표: 옵션 커서 이동
            KeyCode::Down => {
                if !qs.is_custom_input_mode {
                    let max = qs.total_options().saturating_sub(1);
                    if qs.option_cursor < max {
                        qs.option_cursor += 1;
                    }
                }
            }

            // Enter: 선택 확정 또는 텍스트 제출
            KeyCode::Enter => {
                if qs.is_current_freeform() || qs.is_custom_input_mode {
                    // 주관식 또는 직접 입력 모드: 텍스트 버퍼 내용으로 답변
                    let answer = qs.custom_input.clone();
                    if !answer.is_empty() {
                        let completed = qs.submit_answer(answer);
                        if completed {
                            self.handle_action(action::Action::QuestionnaireCompleted);
                        }
                    }
                } else if let Some(q) = qs.current_question() {
                    let cursor = qs.option_cursor;
                    if q.allow_custom && cursor == q.options.len() {
                        // "직접 입력" 선택: 입력 모드 전환
                        qs.is_custom_input_mode = true;
                        qs.custom_input.clear();
                    } else if cursor < q.options.len() {
                        // 객관식 옵션 선택
                        let answer = q.options[cursor].clone();
                        let completed = qs.submit_answer(answer);
                        if completed {
                            self.handle_action(action::Action::QuestionnaireCompleted);
                        }
                    }
                }
            }

            // 문자 입력: 주관식/직접 입력 모드에서 텍스트 버퍼에 추가
            KeyCode::Char(c) => {
                if qs.is_current_freeform() || qs.is_custom_input_mode {
                    qs.custom_input.push(c);
                }
            }

            // Backspace: 주관식/직접 입력 모드에서 문자 삭제
            KeyCode::Backspace => {
                if qs.is_current_freeform() || qs.is_custom_input_mode {
                    qs.custom_input.pop();
                }
            }

            _ => {} // 기타 키는 무시
        }
    }
}
