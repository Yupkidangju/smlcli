pub mod action;
pub mod chat_runtime;
pub mod command_router;
pub mod event_loop;
pub mod state;

use crate::tui::layout::draw;
use crate::tui::terminal::TuiTerminal;
use action::Action;
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
                        match action {
                            action::Action::ToolFinished(res) => {
                                let content = format!(
                                    "[Tool Result] {}\nExit Code: {}\nSTDOUT: {}\nSTDERR: {}",
                                    res.tool_name, res.exit_code, res.stdout, res.stderr
                                );
                                self.state.session.add_message(
                                    crate::providers::types::ChatMessage {
                                        role: crate::providers::types::Role::Tool,
                                        content,
                                        pinned: false,
                                    },
                                );
                            }
                            action::Action::ToolError(e) => {
                                self.state.session.add_message(
                                    crate::providers::types::ChatMessage {
                                        role: crate::providers::types::Role::Tool,
                                        content: format!("[Tool Execution Failed] {}", e),
                                        pinned: false,
                                    },
                                );
                            }
                            action::Action::ChatResponseOk(res) => {
                                self.state.session.token_budget_used +=
                                    res.input_tokens + res.output_tokens;
                                self.state.session.add_message(res.message.clone());

                                // JSON 파서 연동 (```json ... ``` 추출)
                                let content = &res.message.content;
                                if let Some(start_idx) = content.find("```json") {
                                    let block = &content[start_idx + 7..];
                                    if let Some(end_idx) = block.find("```") {
                                        let json_str = block[..end_idx].trim();
                                        if let Ok(tool_call) =
                                            serde_json::from_str::<
                                                crate::domain::tool_result::ToolCall,
                                            >(json_str)
                                        {
                                            // Permission Policy Check
                                            let settings =
                                                self.state.settings.clone().unwrap_or_default();
                                            let perm =
                                                crate::domain::permissions::PermissionEngine::check(
                                                    &tool_call, &settings,
                                                );

                                            match perm {
                                                crate::domain::permissions::PermissionResult::Allow => {
                                                    // 자동 실행
                                                    let tx = self.action_tx.clone();
                                                    let token = crate::domain::permissions::PermissionToken::grant();
                                                    let tool = tool_call.clone();

                                                    tokio::spawn(async move {
                                                        match crate::tools::executor::execute_tool(tool, &token).await {
                                                            Ok(res) => { let _ = tx.send(event_loop::Event::Action(action::Action::ToolFinished(res))).await; }
                                                            Err(e) => { let _ = tx.send(event_loop::Event::Action(action::Action::ToolError(e.to_string()))).await; }
                                                        }
                                                    });
                                                }
                                                crate::domain::permissions::PermissionResult::Ask => {
                                                    self.state.approval.pending_tool = Some(tool_call.clone());
                                                    self.state.show_inspector = true; // 강제 우측 패널 오픈

                                                    // Diff Preview 자동 매핑
                                                    match tool_call {
                                                        crate::domain::tool_result::ToolCall::ReplaceFileContent { path, target_content, replacement_content } => {
                                                            let old_text = std::fs::read_to_string(&path).unwrap_or_default();
                                                            let diff = crate::tools::file_ops::generate_diff(&old_text, &old_text.replace(&target_content, &replacement_content));
                                                            self.state.approval.diff_preview = Some(diff);
                                                        }
                                                        crate::domain::tool_result::ToolCall::WriteFile { path, content, .. } => {
                                                            let diff = crate::tools::file_ops::write_file_preview(&path, &content).unwrap_or_default();
                                                            self.state.approval.diff_preview = Some(diff);
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                crate::domain::permissions::PermissionResult::Deny(reason) => {
                                                    self.state.session.add_message(crate::providers::types::ChatMessage {
                                                        role: crate::providers::types::Role::System,
                                                        content: format!("[Security Block] {}", reason),
                                                        pinned: false,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            action::Action::ChatResponseErr(e) => {
                                self.state.session.add_message(
                                    crate::providers::types::ChatMessage {
                                        role: crate::providers::types::Role::System,
                                        content: format!("Provider Error: {}", e),
                                        pinned: false,
                                    },
                                );
                            }
                            action::Action::ModelsFetched(res) => {
                                if self.state.config.is_open {
                                    self.state.config.is_loading = false;
                                    match res {
                                        Ok(models) => {
                                            self.state.config.available_models = models;
                                            self.state.config.cursor_index = 0;
                                            self.state.config.err_msg = None;
                                        }
                                        Err(e) => {
                                            self.state.config.err_msg = Some(e);
                                        }
                                    }
                                } else {
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
                            // [v0.1.0-beta.7] C-1: API 키 검증 결과 처리
                            action::Action::CredentialValidated(res) => {
                                match res {
                                    Ok(()) => {
                                        // 검증 성공: 이제 fetch_models 진행
                                        self.state.wizard.step = state::WizardStep::ModelSelection;
                                        self.state.wizard.is_loading_models = true;
                                        self.state.wizard.cursor_index = 0;

                                        let tx = self.action_tx.clone();
                                        let provider =
                                            self.state.wizard.selected_provider.clone().unwrap_or(
                                                crate::domain::provider::ProviderKind::OpenRouter,
                                            );
                                        let api_key = self.state.wizard.api_key_input.clone();

                                        tokio::spawn(async move {
                                            let adapter =
                                                crate::providers::registry::get_adapter(&provider);
                                            match adapter.fetch_models(&api_key).await {
                                                Ok(models) => {
                                                    let _ = tx
                                                        .send(event_loop::Event::Action(
                                                            action::Action::ModelsFetched(Ok(
                                                                models,
                                                            )),
                                                        ))
                                                        .await;
                                                }
                                                Err(e) => {
                                                    let _ = tx
                                                        .send(event_loop::Event::Action(
                                                            action::Action::ModelsFetched(Err(
                                                                e.to_string()
                                                            )),
                                                        ))
                                                        .await;
                                                }
                                            }
                                        });
                                    }
                                    Err(e) => {
                                        // 검증 실패: ApiKeyInput 단계로 복귀하고 에러 표시
                                        self.state.wizard.is_loading_models = false;
                                        self.state.wizard.step = state::WizardStep::ApiKeyInput;
                                        self.state.wizard.err_msg =
                                            Some(format!("API 키 검증 실패: {}", e));
                                    }
                                }
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
                    Event::Tick => {
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

    fn handle_input(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};
        match key.code {
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
                        self.state.config.active_popup = state::ConfigPopup::Dashboard;
                    } else {
                        self.state.config.is_open = false;
                    }
                } else {
                    self.state.should_quit = true;
                }
            }
            KeyCode::Char(c) => {
                if self.state.approval.pending_tool.is_some() {
                    if c == 'y' {
                        let tool = self.state.approval.pending_tool.take().unwrap();
                        self.state.approval.diff_preview = None;

                        let tx = self.action_tx.clone();
                        let token = crate::domain::permissions::PermissionToken::grant();

                        // 비동기 실행으로 TUI 프리징 방지
                        tokio::spawn(async move {
                            match crate::tools::executor::execute_tool(tool, &token).await {
                                Ok(res) => {
                                    let _ = tx
                                        .send(event_loop::Event::Action(
                                            action::Action::ToolFinished(res),
                                        ))
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

                        self.state
                            .session
                            .add_message(crate::providers::types::ChatMessage {
                                role: crate::providers::types::Role::System,
                                content: "Tool is running in background...".to_string(),
                                pinned: false,
                            });
                    } else if c == 'n' {
                        self.state.approval.pending_tool = None;
                        self.state.approval.diff_preview = None;
                        self.state
                            .session
                            .add_message(crate::providers::types::ChatMessage {
                                role: crate::providers::types::Role::System,
                                content: "Tool execution rejected by user.".to_string(),
                                pinned: false,
                            });
                    }
                    // [v0.1.0-beta.7] M-1: WizardStep::Home 디버그 점프 제거됨 (삭제된 variant)
                } else if self.state.is_wizard_open {
                    if self.state.wizard.step == state::WizardStep::ApiKeyInput {
                        self.state.wizard.api_key_input.push(c);
                    }
                } else if !self.state.is_wizard_open {
                    if self.state.fuzzy.is_open {
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
            }
            KeyCode::Up => {
                if self.state.fuzzy.is_open {
                    if self.state.fuzzy.cursor > 0 {
                        self.state.fuzzy.cursor -= 1;
                    }
                } else if self.state.config.is_open && self.state.config.cursor_index > 0 {
                    // [v0.1.0-beta.7] H-1: Config 팝업 Up 키 처리
                    self.state.config.cursor_index -= 1;
                } else if self.state.is_wizard_open && self.state.wizard.cursor_index > 0 {
                    self.state.wizard.cursor_index -= 1;
                }
            }
            KeyCode::Down => {
                if self.state.fuzzy.is_open {
                    if self.state.fuzzy.cursor + 1 < self.state.fuzzy.matches.len().min(3) {
                        self.state.fuzzy.cursor += 1;
                    }
                } else if self.state.config.is_open {
                    // [v0.1.0-beta.7] H-1: Config 팝업 Down 키 처리
                    let max = match self.state.config.active_popup {
                        state::ConfigPopup::Dashboard => 2, // Provider, Model, ShellPolicy
                        state::ConfigPopup::ProviderList => 1, // OpenRouter, Google
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
            KeyCode::Backspace => {
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
            KeyCode::Enter => {
                if self.state.fuzzy.is_open {
                    if !self.state.fuzzy.matches.is_empty() {
                        let selected = &self.state.fuzzy.matches[self.state.fuzzy.cursor];
                        self.state
                            .composer
                            .input_buffer
                            .push_str(&format!("@{} ", selected));
                    }
                    self.state.fuzzy.is_open = false;
                } else if self.state.config.is_open {
                    // [v0.1.0-beta.7] H-1: Config 팝업 Enter 키 처리 — 실제 조작 로직 구현
                    match self.state.config.active_popup {
                        state::ConfigPopup::Dashboard => {
                            match self.state.config.cursor_index {
                                0 => {
                                    // Provider 변경 진입
                                    self.state.config.active_popup =
                                        state::ConfigPopup::ProviderList;
                                    self.state.config.cursor_index = 0;
                                }
                                1 => {
                                    // Model 변경 진입 (모델 목록 로딩)
                                    self.state.config.active_popup = state::ConfigPopup::ModelList;
                                    self.state.config.cursor_index = 0;
                                    self.state.config.is_loading = true;
                                    let tx = self.action_tx.clone();
                                    let provider = if let Some(s) = &self.state.settings {
                                        match s.default_provider.as_str() {
                                            "Google" => {
                                                crate::domain::provider::ProviderKind::Google
                                            }
                                            _ => crate::domain::provider::ProviderKind::OpenRouter,
                                        }
                                    } else {
                                        crate::domain::provider::ProviderKind::OpenRouter
                                    };
                                    let api_key = if let Some(s) = &self.state.settings {
                                        crate::infra::secret_store::get_api_key(&format!(
                                            "{}_key",
                                            s.default_provider.to_lowercase()
                                        ))
                                        .unwrap_or_default()
                                    } else {
                                        String::new()
                                    };
                                    tokio::spawn(async move {
                                        let adapter =
                                            crate::providers::registry::get_adapter(&provider);
                                        match adapter.fetch_models(&api_key).await {
                                            Ok(models) => {
                                                let _ = tx
                                                    .send(event_loop::Event::Action(
                                                        action::Action::ModelsFetched(Ok(models)),
                                                    ))
                                                    .await;
                                            }
                                            Err(e) => {
                                                let _ = tx
                                                    .send(event_loop::Event::Action(
                                                        action::Action::ModelsFetched(Err(
                                                            e.to_string()
                                                        )),
                                                    ))
                                                    .await;
                                            }
                                        }
                                    });
                                }
                                2 => {
                                    // ShellPolicy 토글: Ask → SafeOnly → Deny → Ask
                                    if let Some(s) = &mut self.state.settings {
                                        s.shell_policy = match s.shell_policy {
                                            crate::domain::permissions::ShellPolicy::Ask => {
                                                crate::domain::permissions::ShellPolicy::SafeOnly
                                            }
                                            crate::domain::permissions::ShellPolicy::SafeOnly => {
                                                crate::domain::permissions::ShellPolicy::Deny
                                            }
                                            crate::domain::permissions::ShellPolicy::Deny => {
                                                crate::domain::permissions::ShellPolicy::Ask
                                            }
                                        };
                                        // 변경된 설정 즉시 디스크에 저장
                                        if let Ok(mk) =
                                            crate::infra::secret_store::get_or_create_master_key()
                                        {
                                            let _ = crate::infra::config_store::save_config(&mk, s);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        state::ConfigPopup::ProviderList => {
                            // Provider 선택 및 즉시 반영
                            let new_provider = if self.state.config.cursor_index == 0 {
                                "OpenRouter"
                            } else {
                                "Google"
                            };
                            if let Some(s) = &mut self.state.settings {
                                s.default_provider = new_provider.to_string();
                                if let Ok(mk) =
                                    crate::infra::secret_store::get_or_create_master_key()
                                {
                                    let _ = crate::infra::config_store::save_config(&mk, s);
                                }
                            }
                            self.state.config.active_popup = state::ConfigPopup::Dashboard;
                            self.state.config.cursor_index = 0;
                        }
                        state::ConfigPopup::ModelList => {
                            // Model 선택 및 즉시 반영
                            if !self.state.config.available_models.is_empty() {
                                let selected_model = self.state.config.available_models
                                    [self.state.config.cursor_index]
                                    .clone();
                                if let Some(s) = &mut self.state.settings {
                                    s.default_model = selected_model;
                                    if let Ok(mk) =
                                        crate::infra::secret_store::get_or_create_master_key()
                                    {
                                        let _ = crate::infra::config_store::save_config(&mk, s);
                                    }
                                }
                            }
                            self.state.config.active_popup = state::ConfigPopup::Dashboard;
                            self.state.config.cursor_index = 0;
                        }
                    }
                } else if self.state.is_wizard_open {
                    match self.state.wizard.step {
                        state::WizardStep::ProviderSelection => {
                            self.state.wizard.selected_provider =
                                if self.state.wizard.cursor_index == 0 {
                                    Some(crate::domain::provider::ProviderKind::OpenRouter)
                                } else {
                                    Some(crate::domain::provider::ProviderKind::Google)
                                };
                            self.state.wizard.step = state::WizardStep::ApiKeyInput;
                            self.state.wizard.cursor_index = 0;
                        }
                        state::WizardStep::ApiKeyInput => {
                            // [v0.1.0-beta.7] C-1: fetch_models 전에 반드시 validate_credentials 호출
                            // OpenRouter /api/v1/models는 공개 엔드포인트라 인증 없이도 응답하므로,
                            // 장못된 키도 설정이 "성공"하던 버그를 수정.
                            self.state.wizard.is_loading_models = true;
                            self.state.wizard.err_msg = None;

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
                                match adapter.validate_credentials(&api_key).await {
                                    Ok(()) => {
                                        let _ = tx
                                            .send(event_loop::Event::Action(
                                                action::Action::CredentialValidated(Ok(())),
                                            ))
                                            .await;
                                    }
                                    Err(e) => {
                                        let _ = tx
                                            .send(event_loop::Event::Action(
                                                action::Action::CredentialValidated(Err(
                                                    e.to_string()
                                                )),
                                            ))
                                            .await;
                                    }
                                }
                            });
                        }
                        state::WizardStep::ModelSelection => {
                            if !self.state.wizard.available_models.is_empty() {
                                self.state.wizard.selected_model =
                                    self.state.wizard.available_models
                                        [self.state.wizard.cursor_index]
                                        .clone();
                            }
                            self.state.wizard.step = state::WizardStep::Saving;
                        }
                        state::WizardStep::Saving => {
                            let default_model = if self.state.wizard.selected_model.is_empty() {
                                "auto".to_string()
                            } else {
                                self.state.wizard.selected_model.clone()
                            };
                            let provider_str = match &self.state.wizard.selected_provider {
                                Some(crate::domain::provider::ProviderKind::Google) => {
                                    "Google".to_string()
                                }
                                _ => "OpenRouter".to_string(),
                            };
                            let settings = crate::domain::settings::PersistedSettings {
                                version: 1,
                                default_provider: provider_str,
                                default_model,
                                shell_policy: crate::domain::permissions::ShellPolicy::Ask,
                                file_write_policy:
                                    crate::domain::permissions::FileWritePolicy::AlwaysAsk,
                                network_policy:
                                    crate::domain::permissions::NetworkPolicy::ProviderOnly,
                                safe_commands: None,
                            };

                            if let Ok(mk) = crate::infra::secret_store::get_or_create_master_key() {
                                if !self.state.wizard.api_key_input.is_empty() {
                                    let key_alias =
                                        format!("{}_key", settings.default_provider.to_lowercase());
                                    let _ = crate::infra::secret_store::save_api_key(
                                        &key_alias,
                                        &self.state.wizard.api_key_input,
                                    );
                                }
                                let _ = crate::infra::config_store::save_config(&mk, &settings);
                            }
                            self.state.settings = Some(settings); // 메모리에 반영하여 앱의 구동 상태 보장

                            self.state.is_wizard_open = false;
                        }
                    }
                } else {
                    let text = self.state.composer.input_buffer.trim().to_string();
                    if !text.is_empty() {
                        self.state.composer.input_buffer.clear();

                        if text.starts_with('/') {
                            self.handle_slash_command(&text);
                        } else if let Some(stripped) = text.strip_prefix('!') {
                            let cmd = stripped.trim().to_string();
                            if !cmd.is_empty() {
                                let settings = self.state.settings.clone().unwrap_or_default();
                                let tool_call = crate::domain::tool_result::ToolCall::ExecShell {
                                    command: cmd.clone(),
                                    cwd: None,
                                    safe_to_auto_run: false,
                                };
                                let perm = crate::domain::permissions::PermissionEngine::check(
                                    &tool_call, &settings,
                                );

                                match perm {
                                    crate::domain::permissions::PermissionResult::Allow
                                    | crate::domain::permissions::PermissionResult::Ask => {
                                        // 직접 셸 실행은 Allow가 아닐 경우 항상 Ask 처리됨
                                        if matches!(
                                            perm,
                                            crate::domain::permissions::PermissionResult::Allow
                                        ) {
                                            let tx = self.action_tx.clone();
                                            let token =
                                                crate::domain::permissions::PermissionToken::grant(
                                                );
                                            tokio::spawn(async move {
                                                match crate::tools::executor::execute_tool(
                                                    tool_call, &token,
                                                )
                                                .await
                                                {
                                                    Ok(res) => {
                                                        let _ = tx
                                                            .send(event_loop::Event::Action(
                                                                action::Action::ToolFinished(res),
                                                            ))
                                                            .await;
                                                    }
                                                    Err(e) => {
                                                        let _ = tx
                                                            .send(event_loop::Event::Action(
                                                                action::Action::ToolError(
                                                                    e.to_string(),
                                                                ),
                                                            ))
                                                            .await;
                                                    }
                                                }
                                            });
                                        } else {
                                            self.state.approval.pending_tool = Some(tool_call);
                                            self.state.show_inspector = true;
                                        }
                                    }
                                    crate::domain::permissions::PermissionResult::Deny(reason) => {
                                        self.state.session.add_message(
                                            crate::providers::types::ChatMessage {
                                                role: crate::providers::types::Role::System,
                                                content: format!("[Security Block] {}", reason),
                                                pinned: false,
                                            },
                                        );
                                    }
                                }
                            }
                        } else {
                            // [v0.1.0-beta.7] Phase 3: 채팅 로직을 chat_runtime 모듈로 위임
                            self.dispatch_chat_request(text);
                        }
                    }
                }
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

    // [v0.1.0-beta.7] Phase 3: update_fuzzy_matches는 mod.rs에 유지 (UI 상태 밀접 연관)
    fn update_fuzzy_matches(&mut self) {
        let input = self.state.fuzzy.input.clone();

        let mut matches = Vec::new();
        // MVP: 현재 디렉터리 파일 목록 나열. 향후 재귀적 탐색 및 스코어링 알고리즘 적용 예정.
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
