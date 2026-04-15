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
    pub fn new(tx: tokio::sync::mpsc::Sender<event_loop::Event>) -> Self {
        Self {
            state: AppState::new(),
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
                    }
                    Event::Action(action) => {
                        self.handle_action(action);
                    }
                    Event::Tick => {
                        // 컨텍스트 자동 압축 트리거
                        if self.state.session.needs_auto_compaction {
                            self.state.session.needs_auto_compaction = false;
                            self.handle_slash_command("/compact");
                        }
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
            action::Action::ToolFinished(res) => {
                let content = format!(
                    "[Tool Result] {}\nExit Code: {}\nSTDOUT: {}\nSTDERR: {}",
                    res.tool_name, res.exit_code, res.stdout, res.stderr
                );
                self.state
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::Tool,
                        content,
                        pinned: false,
                    });
            }
            action::Action::ToolError(e) => {
                self.state
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::Tool,
                        content: format!("[Tool Execution Failed] {}", e),
                        pinned: false,
                    });
            }
            action::Action::ChatResponseOk(res) => {
                // 토큰 예산 갱신
                self.state.session.token_budget_used += res.input_tokens + res.output_tokens;
                self.state.session.add_message(res.message.clone());

                // 응답에서 도구 호출 감지 및 처리 (tool_runtime.rs에 위임)
                self.process_tool_calls_from_response(&res.message.content);
            }
            action::Action::ChatResponseErr(e) => {
                self.state
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: format!("Provider Error: {}", e),
                        pinned: false,
                    });
            }
            action::Action::ModelsFetched(res, source) => {
                self.handle_models_fetched(res, source);
            }
            action::Action::CredentialValidated(res) => {
                self.handle_credential_validated(res);
            }
            action::Action::ContextSummaryOk(summary) => {
                self.state.session.apply_summary(&summary);
            }
            action::Action::ContextSummaryErr(e) => {
                self.state
                    .session
                    .apply_summary(&format!("Fallback due to error: {}", e));
            }
        }
    }

    /// [v0.1.0-beta.10] ModelsFetched 이벤트 처리: FetchSource에 따라 정확한 상태 슬롯으로 라우팅.
    /// 이전에는 config.is_open으로 판별하여, 팝업을 닫으면 결과가 wizard로 잘못 흐르는 결함이 있었음.
    fn handle_models_fetched(
        &mut self,
        res: Result<Vec<String>, String>,
        source: action::FetchSource,
    ) {
        match source {
            action::FetchSource::Config => {
                self.state.config.is_loading = false;
                match res {
                    Ok(models) => {
                        self.state.config.available_models = models;
                        self.state.config.cursor_index = 0;
                        self.state.config.err_msg = None;
                        // [v0.1.0-beta.12] 8차 감사 H-1: rollback 스냅샷을 여기서 해제하면 안 됨.
                        // 모델 목록 로드 성공 ≠ 사용자 선택 완료. Esc 취소가 가능하므로
                        // rollback은 ModelList 선택이 완료되는 시점까지 유지해야 함.
                    }
                    Err(e) => {
                        self.state.config.err_msg = Some(e);
                        // [v0.1.0-beta.10] 6차 감사 H-1: 검증 실패 시 in-memory 설정 롤백
                        if let (Some(old_p), Some(old_m)) = (
                            self.state.config.rollback_provider.take(),
                            self.state.config.rollback_model.take(),
                        ) && let Some(s) = &mut self.state.settings
                        {
                            s.default_provider = old_p;
                            s.default_model = old_m;
                        }
                        self.state.config.active_popup = state::ConfigPopup::Dashboard;
                    }
                }
            }
            action::FetchSource::Wizard => {
                self.state.wizard.is_loading_models = false;
                match res {
                    Ok(models) => {
                        self.state.wizard.available_models = models;
                        self.state.wizard.cursor_index = 0;
                        self.state.wizard.err_msg = None;
                    }
                    Err(e) => {
                        self.state.wizard.err_msg = Some(e);
                    }
                }
            }
        }
    }

    /// CredentialValidated 이벤트 처리: 검증 성공 시 모델 목록 조회 진행, 실패 시 에러 표시.
    fn handle_credential_validated(&mut self, res: Result<(), String>) {
        match res {
            Ok(()) => {
                // 검증 성공: 이제 fetch_models 진행
                self.state.wizard.step = state::WizardStep::ModelSelection;
                self.state.wizard.is_loading_models = true;
                self.state.wizard.cursor_index = 0;

                let tx = self.action_tx.clone();
                let provider = self
                    .state
                    .wizard
                    .selected_provider
                    .clone()
                    .unwrap_or(crate::domain::provider::ProviderKind::OpenRouter);
                let api_key = self.state.wizard.api_key_input.clone();

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
                            let _ = tx
                                .send(event_loop::Event::Action(action::Action::ModelsFetched(
                                    Err(e.to_string()),
                                    action::FetchSource::Wizard,
                                )))
                                .await;
                        }
                    }
                });
            }
            Err(e) => {
                // 검증 실패: ApiKeyInput 단계로 복귀하고 에러 표시
                self.state.wizard.is_loading_models = false;
                self.state.wizard.step = state::WizardStep::ApiKeyInput;
                self.state.wizard.err_msg = Some(format!("API 키 검증 실패: {}", e));
            }
        }
    }

    /// 키보드 입력 이벤트 처리: 전역 단축키, 승인 UI, 위자드, Fuzzy Finder, Composer를 순차 라우팅.
    fn handle_input(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};
        match key.code {
            // 전역 단축키: Ctrl+C 종료, Ctrl+I 인스펙터 토글
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.should_quit = true;
            }
            KeyCode::Char('i') | KeyCode::Char('I')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.state.show_inspector = !self.state.show_inspector;
            }
            KeyCode::Esc => {
                if self.state.fuzzy.is_open {
                    self.state.fuzzy.is_open = false;
                } else if self.state.config.is_open {
                    if self.state.config.active_popup != state::ConfigPopup::Dashboard {
                        // [v0.1.0-beta.11] 7차 감사 H-2: 사용자 취소 시 롤백.
                        // ProviderList→ModelList 진행 중 Esc로 돌아오면,
                        // in-memory settings를 이전 provider/model로 복구해야 함.
                        if let (Some(old_p), Some(old_m)) = (
                            self.state.config.rollback_provider.take(),
                            self.state.config.rollback_model.take(),
                        ) && let Some(s) = &mut self.state.settings
                        {
                            s.default_provider = old_p;
                            s.default_model = old_m;
                        }
                        self.state.config.err_msg = None;
                        self.state.config.active_popup = state::ConfigPopup::Dashboard;
                    } else {
                        self.state.config.is_open = false;
                        self.state.config.err_msg = None;
                    }
                } else if self.state.is_wizard_open && self.state.wizard.err_msg.is_some() {
                    // [v0.1.0-beta.8] M-1: Error state에서 Esc 시 Wizard 홈으로 회귀
                    self.state.wizard.step = state::WizardStep::ProviderSelection;
                    self.state.wizard.err_msg = None;
                    self.state.wizard.api_key_input.clear();
                    self.state.wizard.cursor_index = 0;
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
                self.handle_enter_key();
            }
            KeyCode::Tab => {
                if !self.state.is_wizard_open {
                    use crate::domain::session::AppMode;
                    self.state.session.mode = match self.state.session.mode {
                        AppMode::Plan => AppMode::Run,
                        AppMode::Run => AppMode::Plan,
                    };
                }
            }
            _ => {}
        }
    }

    /// 문자 입력 처리: 승인 대기 → 위자드 → Fuzzy Finder → Composer 순으로 라우팅.
    fn handle_char_input(&mut self, c: char) {
        if self.state.approval.pending_tool.is_some() {
            if c == 'y' {
                self.handle_tool_approval(true);
            } else if c == 'n' {
                self.handle_tool_approval(false);
            }
        } else if self.state.is_wizard_open {
            if self.state.wizard.step == state::WizardStep::ApiKeyInput {
                self.state.wizard.api_key_input.push(c);
            }
        } else if self.state.fuzzy.is_open {
            self.state.fuzzy.input.push(c);
            self.update_fuzzy_matches();
        } else {
            if c == '@' {
                self.state.fuzzy.is_open = true;
                self.state.fuzzy.input.clear();
                self.state.fuzzy.matches.clear();
                self.state.fuzzy.cursor = 0;
                self.update_fuzzy_matches();
            } else {
                self.state.composer.input_buffer.push(c);
            }
        }
    }

    /// Up 화살표 키 처리: Fuzzy Finder, Config, Wizard 각 모드별 커서 이동.
    fn handle_up_key(&mut self) {
        if self.state.fuzzy.is_open {
            if self.state.fuzzy.cursor > 0 {
                self.state.fuzzy.cursor -= 1;
            }
        } else if self.state.config.is_open && self.state.config.cursor_index > 0 {
            self.state.config.cursor_index -= 1;
        } else if self.state.is_wizard_open && self.state.wizard.cursor_index > 0 {
            self.state.wizard.cursor_index -= 1;
        }
    }

    /// Down 화살표 키 처리: 각 모드별 리스트의 최대값까지 커서 이동.
    fn handle_down_key(&mut self) {
        if self.state.fuzzy.is_open {
            if self.state.fuzzy.cursor + 1 < self.state.fuzzy.matches.len().min(3) {
                self.state.fuzzy.cursor += 1;
            }
        } else if self.state.config.is_open {
            let max = match self.state.config.active_popup {
                state::ConfigPopup::Dashboard => 2,
                state::ConfigPopup::ProviderList => 1,
                state::ConfigPopup::ModelList => {
                    self.state.config.available_models.len().saturating_sub(1)
                }
            };
            if self.state.config.cursor_index < max {
                self.state.config.cursor_index += 1;
            }
        } else if self.state.is_wizard_open {
            let max = match self.state.wizard.step {
                state::WizardStep::ProviderSelection => 1,
                state::WizardStep::ModelSelection => {
                    self.state.wizard.available_models.len().saturating_sub(1)
                }
                _ => 0,
            };
            if self.state.wizard.cursor_index < max {
                self.state.wizard.cursor_index += 1;
            }
        }
    }

    /// Backspace 키 처리: Fuzzy Finder, Wizard API 키, Composer 각각의 버퍼 삭제.
    fn handle_backspace(&mut self) {
        if self.state.fuzzy.is_open {
            if self.state.fuzzy.input.is_empty() {
                self.state.fuzzy.is_open = false;
            } else {
                self.state.fuzzy.input.pop();
                self.update_fuzzy_matches();
            }
        } else if self.state.is_wizard_open {
            if self.state.wizard.step == state::WizardStep::ApiKeyInput {
                self.state.wizard.api_key_input.pop();
            }
        } else {
            self.state.composer.input_buffer.pop();
        }
    }

    /// Enter 키 처리: Fuzzy Finder 선택 → Config 팝업 → Wizard → Composer 제출 순으로 라우팅.
    fn handle_enter_key(&mut self) {
        if self.state.fuzzy.is_open {
            // Fuzzy Finder에서 파일 선택 후 Composer에 참조 주입
            if !self.state.fuzzy.matches.is_empty() {
                let selected = &self.state.fuzzy.matches[self.state.fuzzy.cursor];
                self.state
                    .composer
                    .input_buffer
                    .push_str(&format!("@{} ", selected));
            }
            self.state.fuzzy.is_open = false;
        } else if self.state.config.is_open {
            // Config 팝업 Enter 처리 (wizard_controller.rs에 위임)
            self.handle_config_enter();
        } else if self.state.is_wizard_open {
            // 위자드 Enter 처리 (wizard_controller.rs에 위임)
            self.handle_wizard_enter();
        } else {
            // Composer 제출: 슬래시 커맨드, 직접 셸, 자연어 입력 분기
            let text = self.state.composer.input_buffer.trim().to_string();
            if !text.is_empty() {
                self.state.composer.input_buffer.clear();

                if text.starts_with('/') {
                    self.handle_slash_command(&text);
                } else if let Some(stripped) = text.strip_prefix('!') {
                    self.handle_direct_shell_execution(stripped.trim().to_string());
                } else {
                    self.dispatch_chat_request(text);
                }
            }
        }
    }

    /// Fuzzy Finder 매칭 로직: 현재 디렉터리의 파일 목록을 입력 문자열로 필터링.
    /// 향후 재귀적 탐색 및 스코어링 알고리즘(Levenshtein, Substring) 적용 예정.
    fn update_fuzzy_matches(&mut self) {
        let input = self.state.fuzzy.input.clone();

        let mut matches = Vec::new();
        if let Ok(entries) = std::fs::read_dir(".") {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if !file_type.is_file() {
                        continue;
                    }
                    let name = entry.file_name().into_string().unwrap_or_default();
                    if input.is_empty() || name.to_lowercase().contains(&input.to_lowercase()) {
                        matches.push(name);
                    }
                }
            }
        }

        matches.sort();
        self.state.fuzzy.matches = matches;
        self.state.fuzzy.cursor = 0;
    }
}
