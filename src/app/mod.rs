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

pub mod action;
pub mod chat_runtime;
pub mod command_router;
pub mod event_loop;
pub mod state;
pub mod tool_runtime;
pub mod wizard_controller;

use crate::tui::layout::draw;
use crate::tui::terminal::TuiTerminal;
use anyhow::Result;
use event_loop::{Event, EventLoop};
use state::AppState;

pub struct App {
    pub state: AppState,
    pub action_tx: tokio::sync::mpsc::Sender<event_loop::Event>,
}

impl App {
    pub async fn new(tx: tokio::sync::mpsc::Sender<event_loop::Event>) -> Self {
        Self {
            state: AppState::new_async().await,
            action_tx: tx,
        }
    }

    /// 메인 이벤트 루프. 매 틱마다 UI를 렌더링하고 이벤트를 디스패치.
    /// Input 이벤트는 handle_input으로, Action 이벤트는 handle_action으로 라우팅.
    pub async fn run(
        &mut self,
        terminal: &mut TuiTerminal,
        mut event_loop: EventLoop,
    ) -> Result<()> {
        loop {
            // UI 그리기
            terminal.draw(|f| {
                draw(f, &self.state);
            })?;

            // 이벤트 처리
            if let Ok(event) = event_loop.next().await {
                match event {
                    Event::Quit => {
                        self.state.should_quit = true;
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

                        // [Phase 15-E] follow_tail일 때 커서를 최신 블록으로 동기화
                        if self.state.ui.timeline_follow_tail && !self.state.ui.timeline.is_empty() {
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

        Ok(())
    }

    /// 비동기 액션 이벤트를 각 도메인별 핸들러로 라우팅.
    /// 도구 결과, 채팅 응답, 모델 목록, 인증 결과, 컨텍스트 요약 등 처리.
    fn handle_action(&mut self, action: action::Action) {


        match action {
            // === [v0.1.0-beta.18] 채팅 라이프사이클 ===
            action::Action::ChatStarted => {
                // thinking indicator 시작 + 블록 상태 갱신
                self.state.runtime.is_thinking = true;
                if let Some(block) = self.state.ui.timeline.last_mut() {
                    block.status = crate::app::state::BlockStatus::Running;
                    block.body.push(crate::app::state::BlockSection::Markdown(String::new()));
                }
            }

            action::Action::ChatDelta(token) => {
                // SSE 토큰 수신: 스트리밍 중간 결과에 append
                self.state.runtime.is_thinking = false;
                if let Some(block) = self.state.ui.timeline.last_mut()
                    && let Some(crate::app::state::BlockSection::Markdown(buf)) = block.body.last_mut()
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
                if let Some(block) = self.state.ui.timeline.last_mut() {
                    block.status = crate::app::state::BlockStatus::Done;
                    if let Some(crate::app::state::BlockSection::Markdown(buf)) = block.body.last_mut() {
                        *buf = res.message.content.clone().unwrap_or_default();
                    } else {
                        block.body.push(crate::app::state::BlockSection::Markdown(res.message.content.clone().unwrap_or_default()));
                    }
                }

                // [v0.1.0-beta.22] assistant_turn_count 삭제됨 (재감사 6차).
                // 사유: 첫 턴 차단 로직이 제거되어 카운터만 증가하는 데드 코드였음.

                // 응답에서 도구 호출 감지 및 처리 (tool_runtime.rs에 위임)
                self.process_tool_calls_from_response(&res.message);
            }

            action::Action::ChatResponseErr(e) => {
                // [v0.1.0-beta.16] 추론 완료: thinking indicator 비활성화
                self.state.runtime.is_thinking = false;

                // 스트리밍 Delta가 있으면 에러 메시지로 변환
                if let Some(block) = self.state.ui.timeline.last_mut() {
                    block.status = crate::app::state::BlockStatus::Error;
                    block.body.push(crate::app::state::BlockSection::Markdown(format!("Provider Error: {}", e)));
                } else {
                    let mut block = crate::app::state::TimelineBlock::new(
                        crate::app::state::TimelineBlockKind::Notice,
                        "Provider Error",
                    );
                    block.status = crate::app::state::BlockStatus::Error;
                    block.body.push(crate::app::state::BlockSection::Markdown(e.to_string()));
                    self.state.ui.timeline.push(block);
                }

                self.state
                    .domain
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: Some(format!("Provider Error: {}", e)),
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
                );
                block.status = crate::app::state::BlockStatus::Idle;
                block.body.push(crate::app::state::BlockSection::Markdown("권한 검사 중...".to_string()));
                self.state.ui.timeline.push(block);
            }

            action::Action::ToolStarted(name) => {
                // 마지막 ToolRun 블록의 상태를 Running으로 갱신
                for block in self.state.ui.timeline.iter_mut().rev() {
                    if block.kind == crate::app::state::TimelineBlockKind::ToolRun
                        && (block.title == name || block.status == crate::app::state::BlockStatus::Idle)
                    {
                        block.status = crate::app::state::BlockStatus::Running;
                        break;
                    }
                }
            }

            action::Action::ToolOutputChunk(chunk) => {
                // 원문 로그를 logs_buffer에 추가 (Inspector Logs 탭)
                self.state.runtime.logs_buffer.push(chunk);
            }

            action::Action::ToolFinished(res) => {
                // LLM 컨텍스트에 도구 결과 추가
                let content = format!(
                    "[Tool Result] {}\nExit Code: {}\nSTDOUT: {}\nSTDERR: {}",
                    res.tool_name, res.exit_code, res.stdout, res.stderr
                );
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

                // 원문은 logs_buffer에 보존
                self.state.runtime.logs_buffer.push(format!(
                    "[{}] exit={} stdout={} stderr={}",
                    res.tool_name,
                    res.exit_code,
                    res.stdout.chars().take(500).collect::<String>(),
                    res.stderr.chars().take(500).collect::<String>(),
                ));

                // 타임라인 ToolRun 상태를 Done/Error로 갱신
                let final_status = if res.is_error {
                    crate::app::state::BlockStatus::Error
                } else {
                    crate::app::state::BlockStatus::Done
                };
                for block in self.state.ui.timeline.iter_mut().rev() {
                    if block.kind == crate::app::state::TimelineBlockKind::ToolRun
                        && (block.status == crate::app::state::BlockStatus::Running || block.status == crate::app::state::BlockStatus::Idle)
                    {
                        block.status = final_status.clone();
                        let summary = Self::generate_tool_summary(&res);
                        block.body.push(crate::app::state::BlockSection::ToolSummary {
                            tool_name: res.tool_name.clone(),
                            summary,
                        });
                        break;
                    }
                }

                // [v0.1.0-beta.18] Phase 10: 도구 결과를 LLM에 자동 재전송 (Structured Tool Loop).
                // 도구 실행 결과를 LLM이 해석하여 후속 응답/추가 도구 호출을 자동으로 수행.
                self.send_chat_message_internal();
            }

            action::Action::ToolSummaryReady(new_summary) => {
                // 외부에서 생성된 요약으로 마지막 ToolRun 갱신
                for block in self.state.ui.timeline.iter_mut().rev() {
                    if block.kind == crate::app::state::TimelineBlockKind::ToolRun
                        && let Some(crate::app::state::BlockSection::ToolSummary { summary, .. }) = block.body.last_mut()
                    {
                        *summary = new_summary;
                        break;
                    }
                }
            }

            action::Action::ToolError(e) => {
                self.state
                    .domain
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::Tool,
                        content: Some(format!("[Tool Execution Failed] {}", e)),
                        tool_calls: None,
                        tool_call_id: None,
                        pinned: false,
                    });

                // 타임라인 ToolRun 상태를 Error로 갱신
                for block in self.state.ui.timeline.iter_mut().rev() {
                    if block.kind == crate::app::state::TimelineBlockKind::ToolRun
                        && (block.status == crate::app::state::BlockStatus::Running || block.status == crate::app::state::BlockStatus::Idle)
                    {
                        block.status = crate::app::state::BlockStatus::Error;
                        block.body.push(crate::app::state::BlockSection::Markdown(format!(
                            "실패: {}",
                            e.to_string().chars().take(80).collect::<String>()
                        )));
                        break;
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
            action::Action::ContextSummaryOk(summary) => {
                self.state.domain.session.apply_summary(&summary);
                let mut block = crate::app::state::TimelineBlock::new(
                    crate::app::state::TimelineBlockKind::Notice,
                    "컨텍스트 압축 완료",
                );
                block.body.push(crate::app::state::BlockSection::Markdown(summary));
                self.state.ui.timeline.push(block);
            }
            action::Action::ContextSummaryErr(e) => {
                self.state
                    .domain
                    .session
                    .apply_summary(&format!("Fallback due to error: {}", e));
            }
        }
    }

    /// [v0.1.0-beta.18] ToolResult에서 2~4줄 요약을 생성.
    /// 타임라인에는 이 요약만 표시하고, 원문은 Logs 탭에 보존.
    fn generate_tool_summary(res: &crate::domain::tool_result::ToolResult) -> String {
        let status_icon = if res.is_error { "❌" } else { "✅" };
        let mut summary = format!("{} {} (exit {})", status_icon, res.tool_name, res.exit_code);

        // stdout이 있으면 첫 2줄만 표시
        if !res.stdout.is_empty() {
            let first_lines: String = res.stdout.lines().take(2).collect::<Vec<_>>().join("\n");
            if !first_lines.is_empty() {
                summary.push_str(&format!("\n   {}", first_lines));
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
            }
        }
    }

    /// 키보드 입력 이벤트 처리: 전역 단축키, 승인 UI, 위자드, Fuzzy Finder, Composer를 순차 라우팅.
    pub(crate) fn handle_input(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};
        match key.code {
            // 전역 단축키: Ctrl+C 종료
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.should_quit = true;
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
                if !self.state.ui.is_wizard_open {
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
                        crate::app::state::FocusedPane::Timeline => {
                            self.state.ui.timeline_scroll = self.state.ui.timeline_scroll.saturating_add(5);
                            self.state.ui.timeline_follow_tail = false;
                        }
                        crate::app::state::FocusedPane::Inspector => {
                            self.state.ui.inspector_scroll = self.state.ui.inspector_scroll.saturating_add(5);
                        }
                        _ => {}
                    }
                }
            }
            KeyCode::PageDown => {
                if !self.state.ui.is_wizard_open
                    && !self.state.ui.fuzzy.is_open
                    && !self.state.ui.config.is_open
                {
                    match self.state.ui.focused_pane {
                        crate::app::state::FocusedPane::Timeline => {
                            self.state.ui.timeline_scroll = self.state.ui.timeline_scroll.saturating_sub(5);
                            if self.state.ui.timeline_scroll == 0 {
                                self.state.ui.timeline_follow_tail = true;
                            }
                        }
                        crate::app::state::FocusedPane::Inspector => {
                            self.state.ui.inspector_scroll = self.state.ui.inspector_scroll.saturating_sub(5);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    /// 문자 입력 처리: 승인 대기 → 위자드 → Slash Menu → Fuzzy Finder → Composer 순으로 라우팅.
    fn handle_char_input(&mut self, c: char) {
        if self.state.runtime.approval.pending_tool.is_some() {
            if c == 'y' {
                self.handle_tool_approval(true);
            } else if c == 'n' {
                self.handle_tool_approval(false);
            }
        } else if self.state.ui.is_wizard_open {
            if self.state.ui.wizard.step == state::WizardStep::ApiKeyInput {
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
            } else {
                self.state.ui.composer.input_buffer.push(c);
            }
        }
    }

    /// Up 화살표 키 처리: Slash Menu, Fuzzy Finder, Config, Wizard 각 모드별 커서 이동.
    fn handle_up_key(&mut self) {
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
            self.state.ui.inspector_scroll = self.state.ui.inspector_scroll.saturating_add(1);
        } else if !self.state.ui.composer.history.is_empty() && self.state.ui.focused_pane == crate::app::state::FocusedPane::Composer {
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
                state::ConfigPopup::Dashboard => 2,
                state::ConfigPopup::ProviderList => 1,
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
                state::WizardStep::ProviderSelection => 1,
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
            self.state.ui.inspector_scroll = self.state.ui.inspector_scroll.saturating_sub(1);
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
                self.state.ui.wizard.api_key_input.pop();
            }
        } else {
            self.state.ui.composer.input_buffer.pop();
        }
    }

    /// 마우스 이벤트 처리 (Phase 14-B)
    pub fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        use crossterm::event::{MouseEventKind, MouseButton};

        match mouse.kind {
            MouseEventKind::ScrollUp => {
                match self.state.ui.focused_pane {
                    crate::app::state::FocusedPane::Timeline => {
                        self.state.ui.timeline_scroll_offset += 1;
                        self.state.ui.timeline_follow_tail = false;
                    }
                    crate::app::state::FocusedPane::Inspector => {
                        self.state.ui.inspector_scroll = self.state.ui.inspector_scroll.saturating_add(1);
                    }
                    _ => {}
                }
            }
            MouseEventKind::ScrollDown => {
                match self.state.ui.focused_pane {
                    crate::app::state::FocusedPane::Timeline => {
                        if self.state.ui.timeline_scroll_offset > 0 {
                            self.state.ui.timeline_scroll_offset -= 1;
                            self.state.ui.timeline_follow_tail = self.state.ui.timeline_scroll_offset == 0;
                        }
                    }
                    crate::app::state::FocusedPane::Inspector => {
                        self.state.ui.inspector_scroll = self.state.ui.inspector_scroll.saturating_sub(1);
                    }
                    _ => {}
                }
            }
            MouseEventKind::Down(MouseButton::Left) => {
                // 클릭을 통한 간단한 포커스 라우팅 (간이 레이아웃 기반)
                // 우측 30%가 인스펙터라고 가정 (단순화)
                // row, column을 기준으로 정확한 레이아웃을 알기 어려우므로, 
                // 향후 ratatui 이벤트 라우팅 확장 전까지는 임시 구현.
            }
            _ => {}
        }
    }

    /// Enter 키 처리: Slash Menu 선택 → Fuzzy Finder 선택 → Config 팝업 → Wizard → Composer 제출 순으로 라우팅.
    fn handle_enter_key(&mut self) {
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
        } else {
            // Composer 제출: 슬래시 커맨드, 직접 셸, 자연어 입력 분기
            let text = self.state.ui.composer.input_buffer.trim().to_string();
            if !text.is_empty() {
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
            if input.is_empty() 
                || cmd.title.to_lowercase().contains(&input)
                || cmd.category.to_string().to_lowercase().contains(&input) 
            {
                results.push(cmd.clone());
            }
        }
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
            label: "Ctrl+K Palette".to_string(),
            emphasized: false,
        });

        self.state.ui.toolbar.chips = chips;
        self.state.ui.toolbar.multiline = self.state.ui.composer.input_buffer.contains('\n');
    }
}
