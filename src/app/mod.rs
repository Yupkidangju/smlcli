pub mod state;
pub mod event_loop;
pub mod action;

use anyhow::Result;
use crate::tui::layout::draw;
use crate::tui::terminal::TuiTerminal;
use state::AppState;
use event_loop::{EventLoop, Event};
use action::Action;

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

    pub async fn run(&mut self, terminal: &mut TuiTerminal, mut event_loop: EventLoop) -> Result<()> {
        
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
                                let content = format!("[Tool Result] {}\nExit Code: {}\nSTDOUT: {}\nSTDERR: {}", res.tool_name, res.exit_code, res.stdout, res.stderr);
                                self.state.session.add_message(crate::providers::types::ChatMessage {
                                    role: crate::providers::types::Role::Tool,
                                    content,
                                    pinned: false,
                                });
                            }
                            action::Action::ToolError(e) => {
                                self.state.session.add_message(crate::providers::types::ChatMessage {
                                    role: crate::providers::types::Role::Tool,
                                    content: format!("[Tool Execution Failed] {}", e),
                                    pinned: false,
                                });
                            }
                            action::Action::ChatResponseOk(res) => {
                                self.state.session.token_budget_used += res.input_tokens + res.output_tokens;
                                self.state.session.add_message(res.message.clone());
                                
                                // JSON 파서 연동 (```json ... ``` 추출)
                                let content = &res.message.content;
                                if let Some(start_idx) = content.find("```json") {
                                    let block = &content[start_idx + 7..];
                                    if let Some(end_idx) = block.find("```") {
                                        let json_str = block[..end_idx].trim();
                                        if let Ok(tool_call) = serde_json::from_str::<crate::domain::tool_result::ToolCall>(json_str) {
                                            
                                            // Permission Policy Check
                                            let settings = self.state.settings.clone().unwrap_or_default();
                                            let perm = crate::domain::permissions::PermissionEngine::check(&tool_call, &settings);
                                            
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
                                self.state.session.add_message(crate::providers::types::ChatMessage {
                                    role: crate::providers::types::Role::System,
                                    content: format!("Provider Error: {}", e),
                                    pinned: false,
                                });
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
                                        Err(e) => { self.state.config.err_msg = Some(e); }
                                    }
                                } else {
                                    self.state.wizard.is_loading_models = false;
                                    match res {
                                        Ok(models) => {
                                            self.state.wizard.available_models = models;
                                            self.state.wizard.cursor_index = 0;
                                            self.state.wizard.err_msg = None;
                                        }
                                        Err(e) => { self.state.wizard.err_msg = Some(e); }
                                    }
                                }
                            }
                            action::Action::ContextSummaryOk(summary) => {
                                self.state.session.apply_summary(&summary);
                            }
                            action::Action::ContextSummaryErr(e) => {
                                self.state.session.apply_summary(&format!("Fallback due to error: {}", e));
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
            KeyCode::Char('i') | KeyCode::Char('I') if key.modifiers.contains(KeyModifiers::CONTROL) => {
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
                                Ok(res) => { let _ = tx.send(event_loop::Event::Action(action::Action::ToolFinished(res))).await; }
                                Err(e) => { let _ = tx.send(event_loop::Event::Action(action::Action::ToolError(e.to_string()))).await; }
                            }
                        });
                        
                        self.state.session.add_message(crate::providers::types::ChatMessage {
                            role: crate::providers::types::Role::System,
                            content: "Tool is running in background...".to_string(),
                            pinned: false,
                        });
                    } else if c == 'n' {
                        self.state.approval.pending_tool = None;
                        self.state.approval.diff_preview = None;
                        self.state.session.add_message(crate::providers::types::ChatMessage {
                            role: crate::providers::types::Role::System,
                            content: "Tool execution rejected by user.".to_string(),
                            pinned: false,
                        });
                    }
                } else if self.state.is_wizard_open && self.state.wizard.step == state::WizardStep::Home {
                    match c {
                        '1' => self.state.wizard.step = state::WizardStep::ProviderSelection,
                        '2' => self.state.wizard.step = state::WizardStep::ApiKeyInput,
                        '3' => self.state.wizard.step = state::WizardStep::ModelSelection,
                        '4' => self.state.wizard.step = state::WizardStep::PermissionPreset,
                        '5' => self.state.wizard.step = state::WizardStep::Saving,
                        _ => {}
                    }
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
                } else if self.state.is_wizard_open && self.state.wizard.cursor_index > 0 {
                    self.state.wizard.cursor_index -= 1;
                }
            }
            KeyCode::Down => {
                if self.state.fuzzy.is_open {
                    if self.state.fuzzy.cursor + 1 < self.state.fuzzy.matches.len().min(3) {
                        self.state.fuzzy.cursor += 1;
                    }
                } else if self.state.is_wizard_open {
                    let max = match self.state.wizard.step {
                        state::WizardStep::ProviderSelection => 1,
                        state::WizardStep::ModelSelection => self.state.wizard.available_models.len().saturating_sub(1),
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
                        self.state.composer.input_buffer.push_str(&format!("@{} ", selected));
                    }
                    self.state.fuzzy.is_open = false;
                } else if self.state.is_wizard_open {
                    match self.state.wizard.step {
                        state::WizardStep::ProviderSelection => {
                            self.state.wizard.selected_provider = if self.state.wizard.cursor_index == 0 {
                                Some(crate::domain::provider::ProviderKind::OpenRouter)
                            } else {
                                Some(crate::domain::provider::ProviderKind::Google)
                            };
                            self.state.wizard.step = state::WizardStep::ApiKeyInput;
                            self.state.wizard.cursor_index = 0;
                        }
                        state::WizardStep::ApiKeyInput => {
                            self.state.wizard.is_loading_models = true;
                            self.state.wizard.step = state::WizardStep::ModelSelection;
                            self.state.wizard.cursor_index = 0;
                            
                            let tx = self.action_tx.clone();
                            let provider = self.state.wizard.selected_provider.clone().unwrap_or(crate::domain::provider::ProviderKind::OpenRouter);
                            let api_key = self.state.wizard.api_key_input.clone();
                            
                            tokio::spawn(async move {
                                let adapter = crate::providers::registry::get_adapter(&provider);
                                match adapter.fetch_models(&api_key).await {
                                    Ok(models) => { let _ = tx.send(event_loop::Event::Action(action::Action::ModelsFetched(Ok(models)))).await; }
                                    Err(e) => { let _ = tx.send(event_loop::Event::Action(action::Action::ModelsFetched(Err(e.to_string())))).await; }
                                }
                            });
                        }
                        state::WizardStep::ModelSelection => {
                            if !self.state.wizard.available_models.is_empty() {
                                self.state.wizard.selected_model = self.state.wizard.available_models[self.state.wizard.cursor_index].clone();
                            }
                            self.state.wizard.step = state::WizardStep::Saving;
                        }
                        state::WizardStep::Saving => {
                            let default_model = if self.state.wizard.selected_model.is_empty() { "auto".to_string() } else { self.state.wizard.selected_model.clone() };
                            let provider_str = match &self.state.wizard.selected_provider {
                                Some(crate::domain::provider::ProviderKind::Google) => "Google".to_string(),
                                _ => "OpenRouter".to_string(),
                            };
                            let settings = crate::domain::settings::PersistedSettings {
                                version: 1,
                                default_provider: provider_str,
                                default_model,
                                shell_policy: crate::domain::permissions::ShellPolicy::Ask,
                                file_write_policy: crate::domain::permissions::FileWritePolicy::AlwaysAsk,
                                network_policy: crate::domain::permissions::NetworkPolicy::ProviderOnly,
                                safe_commands: None,
                            };

                            if let Ok(mk) = crate::infra::secret_store::get_or_create_master_key() {
                                if !self.state.wizard.api_key_input.is_empty() {
                                    let key_alias = format!("{}_key", settings.default_provider.to_lowercase());
                                    let _ = crate::infra::secret_store::save_api_key(&key_alias, &self.state.wizard.api_key_input);
                                }
                                let _ = crate::infra::config_store::save_config(&mk, &settings);
                            }
                            self.state.settings = Some(settings); // 메모리에 반영하여 앱의 구동 상태 보장

                            self.state.is_wizard_open = false;
                        }
                        _ => {}
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
                                let perm = crate::domain::permissions::PermissionEngine::check(&tool_call, &settings);

                                match perm {
                                    crate::domain::permissions::PermissionResult::Allow | crate::domain::permissions::PermissionResult::Ask => {
                                        // 직접 셸 실행은 Allow가 아닐 경우 항상 Ask 처리됨
                                        if matches!(perm, crate::domain::permissions::PermissionResult::Allow) {
                                            let tx = self.action_tx.clone();
                                            let token = crate::domain::permissions::PermissionToken::grant();
                                            tokio::spawn(async move {
                                                match crate::tools::executor::execute_tool(tool_call, &token).await {
                                                    Ok(res) => { let _ = tx.send(event_loop::Event::Action(action::Action::ToolFinished(res))).await; }
                                                    Err(e) => { let _ = tx.send(event_loop::Event::Action(action::Action::ToolError(e.to_string()))).await; }
                                                }
                                            });
                                        } else {
                                            self.state.approval.pending_tool = Some(tool_call);
                                            self.state.show_inspector = true;
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
                        } else {
                            let mut final_text = text.clone();
                            if text.contains('@') {
                                let parts: Vec<&str> = text.split_whitespace().collect();
                                for word in parts {
                                    if word.starts_with('@') && word.len() > 1 {
                                        let path = &word[1..];
                                        if let Ok(content) = std::fs::read_to_string(path) {
                                            final_text = final_text.replace(word, &format!("\n--- {} ---\n{}\n--- End of {} ---\n", path, content, path));
                                        }
                                    }
                                }
                            }
                            
                            let msg = crate::providers::types::ChatMessage {
                                role: crate::providers::types::Role::User,
                                content: final_text,
                                pinned: false,
                            };
                            self.state.session.add_message(msg);
                            
                            let tx = self.action_tx.clone();
                            let messages = self.state.session.messages.clone();
                            let settings_clone = self.state.settings.clone();
                            
                            tokio::spawn(async move {
                                let (provider_kind, model_name, api_key) = if let Some(s) = &settings_clone {
                                    let provider = match s.default_provider.as_str() {
                                        "Google" => crate::domain::provider::ProviderKind::Google,
                                        _ => crate::domain::provider::ProviderKind::OpenRouter,
                                    };
                                    let alias = format!("{}_key", s.default_provider.to_lowercase());
                                    let key = crate::infra::secret_store::get_api_key(&alias).unwrap_or_else(|_| "dummy_key".to_string());
                                    (provider, s.default_model.clone(), key)
                                } else {
                                    (crate::domain::provider::ProviderKind::OpenRouter, "auto".to_string(), "dummy_key".to_string())
                                };
                                
                                let adapter = crate::providers::registry::get_adapter(&provider_kind);
                                let req = crate::providers::types::ChatRequest {
                                    model: model_name,
                                    messages,
                                };
                                match adapter.chat(&api_key, req).await {
                                    Ok(res) => { let _ = tx.send(event_loop::Event::Action(action::Action::ChatResponseOk(res))).await; }
                                    Err(e) => { let _ = tx.send(event_loop::Event::Action(action::Action::ChatResponseErr(e.to_string()))).await; }
                                }
                            });
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

    fn handle_slash_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts[0] {
            "/setting" => {
                self.state.is_wizard_open = true;
                self.state.wizard = state::WizardState::new();
            }
            "/config" => {
                self.state.config.is_open = true;
                self.state.config.active_popup = state::ConfigPopup::Dashboard;
                self.state.config.cursor_index = 0;
            }
            "/provider" => {
                self.state.config.is_open = true;
                self.state.config.active_popup = state::ConfigPopup::ProviderList;
                self.state.config.cursor_index = 0;
            }
            "/model" => {
                self.state.config.is_open = true;
                self.state.config.active_popup = state::ConfigPopup::ModelList;
                self.state.config.cursor_index = 0;
                
                self.state.config.is_loading = true;
                let tx = self.action_tx.clone();
                let provider = if let Some(s) = &self.state.settings {
                    match s.default_provider.as_str() {
                        "Google" => crate::domain::provider::ProviderKind::Google,
                        _ => crate::domain::provider::ProviderKind::OpenRouter,
                    }
                } else {
                    crate::domain::provider::ProviderKind::OpenRouter
                };
                let api_key = if let Some(s) = &self.state.settings {
                    crate::infra::secret_store::get_api_key(&format!("{}_key", s.default_provider.to_lowercase())).unwrap_or_default()
                } else {
                    "".to_string()
                };
                
                tokio::spawn(async move {
                    let adapter = crate::providers::registry::get_adapter(&provider);
                    match adapter.fetch_models(&api_key).await {
                        Ok(models) => { let _ = tx.send(event_loop::Event::Action(action::Action::ModelsFetched(Ok(models)))).await; }
                        Err(e) => { let _ = tx.send(event_loop::Event::Action(action::Action::ModelsFetched(Err(e.to_string())))).await; }
                    }
                });
            }
            "/status" => {
                let info = if let Some(s) = &self.state.settings {
                    format!("Provider: {}\nModel: {}\nBudget Used: {} tokens", s.default_provider, s.default_model, self.state.session.token_budget_used)
                } else {
                    "Not configured.".to_string()
                };
                self.state.session.add_message(crate::providers::types::ChatMessage {
                    role: crate::providers::types::Role::System,
                    content: format!("[Status]\n{}", info),
                    pinned: false,
                });
            }
            "/mode" => {
                use crate::domain::session::AppMode;
                self.state.session.mode = match self.state.session.mode {
                    AppMode::Plan => AppMode::Run,
                    AppMode::Run => AppMode::Plan,
                };
            }
            "/clear" => {
                self.state.session.messages.clear();
            }
            "/compact" => {
                let to_summarize = self.state.session.extract_for_summary();
                if to_summarize.is_empty() {
                    self.state.session.add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: "Context too small to compress.".to_string(),
                        pinned: false,
                    });
                } else {
                    let tx = self.action_tx.clone();
                    let settings_clone = self.state.settings.clone();
                    
                    tokio::spawn(async move {
                        let (provider_kind, model_name, api_key) = if let Some(s) = &settings_clone {
                            let provider = match s.default_provider.as_str() {
                                "Google" => crate::domain::provider::ProviderKind::Google,
                                _ => crate::domain::provider::ProviderKind::OpenRouter,
                            };
                            let key = crate::infra::secret_store::get_api_key(&format!("{}_key", s.default_provider.to_lowercase())).unwrap_or_default();
                            (provider, s.default_model.clone(), key)
                        } else {
                            return;
                        };
                        
                        let mut content = "Summarize the following chat context into a brief 3-bullet list to preserve the goals and actions:\n".to_string();
                        for m in to_summarize {
                            let r = match m.role { crate::providers::types::Role::User => "User", _ => "Other" };
                            content.push_str(&format!("{}: {}\n\n", r, m.content));
                        }
                        
                        let req = crate::providers::types::ChatRequest {
                            model: model_name,
                            messages: vec![crate::providers::types::ChatMessage {
                                role: crate::providers::types::Role::User,
                                content,
                                pinned: false,
                            }],
                        };
                        
                        let adapter = crate::providers::registry::get_adapter(&provider_kind);
                        match adapter.chat(&api_key, req).await {
                            Ok(res) => { let _ = tx.send(event_loop::Event::Action(action::Action::ContextSummaryOk(res.message.content))).await; }
                            Err(e) => { let _ = tx.send(event_loop::Event::Action(action::Action::ContextSummaryErr(e.to_string()))).await; }
                        }
                    });
                }
            }
            "/tokens" => {
                let budget = self.state.session.get_context_load_percentage();
                let estimated = self.state.session.estimate_current_tokens();
                let cap = self.state.session.max_token_budget;
                self.state.session.add_message(crate::providers::types::ChatMessage {
                    role: crate::providers::types::Role::System,
                    content: format!("[Tokens Insight]\nEstimated tokens in context: {} / {} ({}%)", estimated, cap, budget),
                    pinned: false,
                });
            }
            "/help" => {
                let help_text = "/config: Settings Dashboard\n/setting: Setup Wizard\n/provider: Switch Provider\n/model: Switch Model\n/status: Show Session Info\n/mode: Toggle PLAN/RUN\n/tokens: Show Token Limits\n/compact: Compress Chat Context\n/clear: Clear Chat\n/help: Show this message\n/quit: Exit";
                self.state.session.add_message(crate::providers::types::ChatMessage {
                    role: crate::providers::types::Role::System,
                    content: help_text.to_string(),
                    pinned: false,
                });
            }
            "/quit" => {
                self.state.should_quit = true;
            }
            _ => {
                self.state.session.add_message(crate::providers::types::ChatMessage {
                    role: crate::providers::types::Role::System,
                    content: format!("Unknown command: {}", parts[0]),
                    pinned: false,
                });
            }
        }
    }

    fn update_fuzzy_matches(&mut self) {
        let input = self.state.fuzzy.input.clone();
        
        let mut matches = Vec::new();
        // MVP: 현재 디렉터리 파일 목록 나열. 실제 구현에서는 재귀적 및 필터링 적용 필요.
        if let Ok(entries) = std::fs::read_dir(".") {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if !file_type.is_file() { continue; }
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
