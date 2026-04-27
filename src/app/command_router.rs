// [v0.1.0-beta.7] Phase 3 리팩토링: mod.rs에서 슬래시 커맨드 엔진 분리.
// 12개의 내부 명령어(/config, /setting, /provider, /model, /status, /mode, /clear, /compact, /tokens, /help, /quit)의
// 파싱과 실행을 전담하는 모듈.
// 이전에는 mod.rs 내 handle_slash_command 메서드에 모든 로직이 인라인되어 있었음.
//
// [v0.1.0-beta.9] 5차 감사: /model과 /compact가 중앙 보안 가드(resolve_credentials)를 우회하던 문제 수정.
// unwrap_or_default()로 빈 키를 삼키던 패턴을 제거하고, NetworkPolicy + 암호화 저장소 검증을 일관 적용.

use super::{App, action, event_loop, state};

impl App {
    /// 사용자 입력이 '/'로 시작할 때 호출되는 슬래시 커맨드 라우터.
    /// 각 커맨드에 대한 상태 변경, 비동기 작업 트리거, 메시지 추가를 수행.
    pub(crate) fn handle_slash_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts[0] {
            "/setting" => {
                self.state.ui.is_wizard_open = true;
                self.state.ui.wizard = state::WizardState::new();
            }
            "/config" => {
                self.state.ui.config.is_open = true;
                self.state.ui.config.active_popup = state::ConfigPopup::Dashboard;
                self.state.ui.config.cursor_index = 0;
            }
            "/provider" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "add" => {
                            if parts.len() < 4 {
                                self.state.domain.session.add_message(crate::providers::types::ChatMessage {
                                    role: crate::providers::types::Role::System,
                                    // [v3.3.1] 감사 MEDIUM-3: Usage 메시지에 auth_header_name 추가
                                    content: Some("Usage: /provider add <id> <base_url> [dialect] [auth_type] [auth_header_name]\nExample: /provider add local-ollama http://localhost:11434/v1 OpenAICompat none\nExample: /provider add custom-api https://api.example.com OpenAICompat CustomHeader X-API-Key\n\nauth_type: Bearer(기본값), none, CustomHeader\nauth_header_name: CustomHeader 사용 시 헤더 이름 (기본값: Authorization)".to_string()),
                                    tool_calls: None,
                                    tool_call_id: None,
                                    pinned: false,
                                });
                                return;
                            }
                            let id = parts[2].to_string();
                            let base_url = parts[3].to_string();
                            let dialect_str = parts.get(4).unwrap_or(&"OpenAICompat").to_string();
                            let dialect = match dialect_str.to_lowercase().as_str() {
                                "anthropic" => crate::domain::provider::ToolDialect::Anthropic,
                                "gemini" => crate::domain::provider::ToolDialect::Gemini,
                                _ => crate::domain::provider::ToolDialect::OpenAICompat,
                            };
                            let auth_type = parts.get(5).unwrap_or(&"Bearer").to_string();
                            // [v3.3.1] 감사 MEDIUM-3 수정: auth_header_name을 6번째 인자로 지정 가능.
                            // CustomHeader auth_type일 때 X-API-Key 등의 비표준 헤더를 사용할 수 있음.
                            // 미지정 시 기본값은 auth_type에 따라 결정:
                            // - "CustomHeader" → 반드시 지정 필요 (없으면 "Authorization" 폴백)
                            // - 그 외 → "Authorization"
                            let auth_header_name = parts
                                .get(6)
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| "Authorization".to_string());

                            let config = crate::domain::provider::CustomProviderConfig {
                                id: id.clone(),
                                base_url,
                                auth_type,
                                auth_header_name: Some(auth_header_name),
                                dialect,
                            };

                            if let Some(settings) = &mut self.state.domain.settings {
                                settings.custom_providers.retain(|p| p.id != id);
                                settings.custom_providers.push(config);
                                crate::providers::registry::update_custom_providers(
                                    &settings.custom_providers,
                                );

                                let settings_clone = settings.clone();
                                let tx = self.action_tx.clone();
                                tokio::spawn(async move {
                                    let res =
                                        crate::infra::config_store::save_config(&settings_clone)
                                            .await
                                            .map_err(|e| e.to_string());
                                    let _ = tx
                                        .send(crate::app::event_loop::Event::Action(
                                            crate::app::action::Action::ConfigSaveFinished(res),
                                        ))
                                        .await;
                                });

                                self.state.domain.session.add_message(
                                    crate::providers::types::ChatMessage {
                                        role: crate::providers::types::Role::System,
                                        content: Some(format!(
                                            "Custom provider '{}' added successfully.",
                                            id
                                        )),
                                        tool_calls: None,
                                        tool_call_id: None,
                                        pinned: false,
                                    },
                                );
                            }
                        }
                        "remove" => {
                            if parts.len() < 3 {
                                self.state.domain.session.add_message(
                                    crate::providers::types::ChatMessage {
                                        role: crate::providers::types::Role::System,
                                        content: Some("Usage: /provider remove <id>".to_string()),
                                        tool_calls: None,
                                        tool_call_id: None,
                                        pinned: false,
                                    },
                                );
                                return;
                            }
                            let id = parts[2].to_string();
                            if let Some(settings) = &mut self.state.domain.settings {
                                let initial_len = settings.custom_providers.len();
                                settings.custom_providers.retain(|p| p.id != id);
                                if settings.custom_providers.len() < initial_len {
                                    crate::providers::registry::reload_providers();
                                    crate::providers::registry::update_custom_providers(
                                        &settings.custom_providers,
                                    );

                                    let settings_clone = settings.clone();
                                    let tx = self.action_tx.clone();
                                    tokio::spawn(async move {
                                        let res = crate::infra::config_store::save_config(
                                            &settings_clone,
                                        )
                                        .await
                                        .map_err(|e| e.to_string());
                                        let _ = tx
                                            .send(crate::app::event_loop::Event::Action(
                                                crate::app::action::Action::ConfigSaveFinished(res),
                                            ))
                                            .await;
                                    });
                                    self.state.domain.session.add_message(
                                        crate::providers::types::ChatMessage {
                                            role: crate::providers::types::Role::System,
                                            content: Some(format!(
                                                "Custom provider '{}' removed.",
                                                id
                                            )),
                                            tool_calls: None,
                                            tool_call_id: None,
                                            pinned: false,
                                        },
                                    );
                                } else {
                                    self.state.domain.session.add_message(
                                        crate::providers::types::ChatMessage {
                                            role: crate::providers::types::Role::System,
                                            content: Some(format!(
                                                "Custom provider '{}' not found.",
                                                id
                                            )),
                                            tool_calls: None,
                                            tool_call_id: None,
                                            pinned: false,
                                        },
                                    );
                                }
                            }
                        }
                        "list" => {
                            if let Some(settings) = &self.state.domain.settings {
                                if settings.custom_providers.is_empty() {
                                    self.state.domain.session.add_message(
                                        crate::providers::types::ChatMessage {
                                            role: crate::providers::types::Role::System,
                                            content: Some(
                                                "No custom providers registered.".to_string(),
                                            ),
                                            tool_calls: None,
                                            tool_call_id: None,
                                            pinned: false,
                                        },
                                    );
                                } else {
                                    let mut s = "Registered Custom Providers:\n".to_string();
                                    for cp in &settings.custom_providers {
                                        s.push_str(&format!(
                                            "- {} (URL: {}, Dialect: {:?})\n",
                                            cp.id, cp.base_url, cp.dialect
                                        ));
                                    }
                                    self.state.domain.session.add_message(
                                        crate::providers::types::ChatMessage {
                                            role: crate::providers::types::Role::System,
                                            content: Some(s),
                                            tool_calls: None,
                                            tool_call_id: None,
                                            pinned: false,
                                        },
                                    );
                                }
                            }
                        }
                        _ => {
                            self.state
                                .domain
                                .session
                                .add_message(crate::providers::types::ChatMessage {
                                role: crate::providers::types::Role::System,
                                content: Some(
                                    "Unknown /provider subcommand. Use 'add', 'remove', or 'list'."
                                        .to_string(),
                                ),
                                tool_calls: None,
                                tool_call_id: None,
                                pinned: false,
                            });
                        }
                    }
                } else {
                    self.state.ui.config.is_open = true;
                    self.state.ui.config.active_popup = state::ConfigPopup::ProviderList;
                    self.state.ui.config.cursor_index = 0;
                }
            }
            "/model" => {
                // [v0.1.0-beta.9] 중앙 보안 가드 적용: NetworkPolicy + 암호화 저장소 검증 후 모델 페칭
                let (provider_kind, _model_name, api_key) = match self.resolve_credentials() {
                    Ok(creds) => creds,
                    Err(err_msg) => {
                        self.state.domain.session.add_message(
                            crate::providers::types::ChatMessage {
                                role: crate::providers::types::Role::System,
                                content: Some(err_msg.to_string()),
                                tool_calls: None,
                                tool_call_id: None,
                                pinned: false,
                            },
                        );
                        return;
                    }
                };

                self.state.ui.config.is_open = true;
                self.state.ui.config.active_popup = state::ConfigPopup::ModelList;
                self.state.ui.config.cursor_index = 0;
                self.state.ui.config.is_loading = true;

                let tx = self.action_tx.clone();
                tokio::spawn(async move {
                    let adapter = crate::providers::registry::get_adapter(&provider_kind);

                    // [v0.1.0-beta.10] 6차 감사 M-1: validate_credentials 선행 검증.
                    // OpenRouter /models는 공개 엔드포인트라 가짜 키도 200 반환하므로,
                    // /auth/key로 키 유효성을 먼저 확인해야 함.
                    if let Err(e) = adapter.validate_credentials(&api_key).await {
                        // [v0.1.0-beta.21] ProviderError 구조화
                        let _ = tx
                            .send(event_loop::Event::Action(action::Action::ModelsFetched(
                                Err(crate::domain::error::ProviderError::AuthenticationFailed(
                                    format!("API key validation failed: {}", e),
                                )),
                                action::FetchSource::Config,
                            )))
                            .await;
                        return;
                    }

                    match adapter.fetch_models(&api_key).await {
                        Ok(models) => {
                            let _ = tx
                                .send(event_loop::Event::Action(action::Action::ModelsFetched(
                                    Ok(models),
                                    action::FetchSource::Config,
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
                                    action::FetchSource::Config,
                                )))
                                .await;
                        }
                    }
                });
            }
            "/status" => {
                let root = std::env::current_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let info = if let Some(s) = &self.state.domain.settings {
                    let trust = s.get_workspace_trust(&root);
                    format!(
                        "Provider: {}\nModel: {}\nBudget Used: {} tokens\nHost Shell: {}\nExec Shell: {}\nWorkspace Trust: {:?}\nDenied: {}",
                        s.default_provider,
                        s.default_model,
                        self.state.domain.session.token_budget_used,
                        self.state.runtime.workspace.host_shell,
                        self.state.runtime.workspace.exec_shell,
                        trust,
                        s.denied_roots.contains(&root)
                    )
                } else {
                    "Not configured.".to_string()
                };
                self.state
                    .domain
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: Some(format!("[Status]\n{}", info)),
                        tool_calls: None,
                        tool_call_id: None,
                        pinned: false,
                    });
            }
            "/mode" => {
                use crate::domain::session::AppMode;
                self.state.domain.session.mode = match self.state.domain.session.mode {
                    AppMode::Plan => AppMode::Run,
                    AppMode::Run => AppMode::Plan,
                };
            }
            "/clear" => {
                // [v0.1.0-beta.7] pinned 메시지(시스템 프롬프트, 요약)를 보존하고 나머지만 삭제.
                self.state.domain.session.messages.retain(|m| m.pinned);
            }
            "/compact" => {
                self.handle_compact_command();
            }
            "/tokens" => {
                let budget = self.state.domain.session.get_context_load_percentage();
                let estimated = self.state.domain.session.estimate_current_tokens();
                let cap = self.state.domain.session.max_token_budget;
                self.state
                    .domain
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: Some(format!(
                            "[Tokens Insight]\nEstimated tokens in context: {} / {} ({}%)",
                            estimated, cap, budget
                        )),
                        tool_calls: None,
                        tool_call_id: None,
                        pinned: false,
                    });
            }
            "/help" => {
                let help_entries = vec![
                    (
                        "/config".to_string(),
                        "설정 대시보드 (Settings Dashboard)".to_string(),
                    ),
                    (
                        "/setting".to_string(),
                        "셋업 위자드 (Setup Wizard)".to_string(),
                    ),
                    (
                        "/provider".to_string(),
                        "공급자 전환 (Switch Provider)".to_string(),
                    ),
                    ("/model".to_string(), "모델 전환 (Switch Model)".to_string()),
                    (
                        "/status".to_string(),
                        "세션 상태 (Session Info)".to_string(),
                    ),
                    (
                        "/mode".to_string(),
                        "PLAN ↔ RUN 전환 (Toggle Mode)".to_string(),
                    ),
                    (
                        "/tokens".to_string(),
                        "토큰 사용량 (Token Usage)".to_string(),
                    ),
                    (
                        "/compact".to_string(),
                        "컨텍스트 압축 (Compress Context)".to_string(),
                    ),
                    ("/theme".to_string(), "테마 전환 (Toggle Theme)".to_string()),
                    (
                        "/workspace".to_string(),
                        "워크스페이스 신뢰 관리 (Manage Workspace Trust)".to_string(),
                    ),
                    (
                        "/mcp".to_string(),
                        "MCP 서버 관리 (list, add, remove)".to_string(),
                    ),
                    (
                        "/undo".to_string(),
                        "마지막 AI 작업 되돌리기 (Undo Last AI Commit)".to_string(),
                    ),
                    ("/new".to_string(), "새 세션 시작 (New Session)".to_string()),
                    (
                        "/resume".to_string(),
                        "세션 이어하기 (Resume Session)".to_string(),
                    ),
                    (
                        "/session".to_string(),
                        "세션 목록 (Session List)".to_string(),
                    ),
                    ("/clear".to_string(), "대화 초기화 (Clear Chat)".to_string()),
                    ("/help".to_string(), "도움말 (Help)".to_string()),
                    ("/quit".to_string(), "종료 (Exit)".to_string()),
                ];
                let mut block = crate::app::state::TimelineBlock::new(
                    crate::app::state::TimelineBlockKind::Help,
                    "도움말",
                );
                block
                    .body
                    .push(crate::app::state::BlockSection::KeyValueTable(help_entries));
                self.state.ui.timeline.push(block);
            }
            "/quit" => {
                self.state.should_quit = true;
            }
            "/theme" => {
                if let Some(settings) = &mut self.state.domain.settings {
                    let new_theme = if settings.theme == "high_contrast" {
                        "default".to_string()
                    } else {
                        "high_contrast".to_string()
                    };
                    settings.theme = new_theme.clone();

                    // 설정 변경을 config.toml에 비동기 저장
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

                    self.state
                        .domain
                        .session
                        .add_message(crate::providers::types::ChatMessage {
                            role: crate::providers::types::Role::System,
                            content: Some(format!(
                                "[Theme] 테마가 '{}'(으)로 전환되었습니다.",
                                new_theme
                            )),
                            tool_calls: None,
                            tool_call_id: None,
                            pinned: false,
                        });
                } else {
                    self.state
                        .domain
                        .session
                        .add_message(crate::providers::types::ChatMessage {
                            role: crate::providers::types::Role::System,
                            content: Some(
                                "설정이 없습니다. /setting으로 초기 설정을 진행하세요.".to_string(),
                            ),
                            tool_calls: None,
                            tool_call_id: None,
                            pinned: false,
                        });
                }
            }
            "/workspace" => {
                if parts.len() < 2 {
                    self.state
                        .domain
                        .session
                        .add_message(crate::providers::types::ChatMessage {
                            role: crate::providers::types::Role::System,
                            content: Some("Usage: /workspace <show|trust|deny|clear>".to_string()),
                            tool_calls: None,
                            tool_call_id: None,
                            pinned: false,
                        });
                    return;
                }

                let root = std::env::current_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let subcmd = parts[1];
                let message;

                if let Some(settings) = &mut self.state.domain.settings {
                    match subcmd {
                        "show" => {
                            let trust = settings.get_workspace_trust(&root);
                            let is_denied = settings.denied_roots.contains(&root);
                            message = format!(
                                "Workspace: {}\nTrust Level: {:?}\nDenied: {}",
                                root, trust, is_denied
                            );
                        }
                        "trust" => {
                            settings.set_workspace_trust(
                                &root,
                                crate::domain::settings::WorkspaceTrustState::Trusted,
                                true,
                            );
                            settings.denied_roots.retain(|x| x != &root);
                            message = format!("Workspace {} is now Trusted.", root);

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
                        "deny" => {
                            settings.set_workspace_trust(
                                &root,
                                crate::domain::settings::WorkspaceTrustState::Restricted,
                                true,
                            );
                            if !settings.denied_roots.contains(&root) {
                                settings.denied_roots.push(root.clone());
                            }
                            message = format!("Workspace {} is now Denied (Restricted).", root);

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
                        "clear" => {
                            settings.remove_workspace_trust(&root);
                            settings.denied_roots.retain(|x| x != &root);
                            message = format!("Workspace {} trust records cleared.", root);

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
                        _ => {
                            message = format!("Unknown workspace command: {}", subcmd);
                        }
                    }
                } else {
                    message = "Settings not available.".to_string();
                }

                self.state
                    .domain
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: Some(message),
                        tool_calls: None,
                        tool_call_id: None,
                        pinned: false,
                    });
            }
            "/mcp" => {
                let mut args_iter = parts.iter().skip(1).copied();
                let subcmd = args_iter.next().unwrap_or("list");

                #[allow(unused_assignments)]
                let mut message = String::new();
                if let Some(settings) = &mut self.state.domain.settings {
                    match subcmd {
                        "list" => {
                            if settings.mcp_servers.is_empty() {
                                message = "No MCP servers configured.".to_string();
                            } else {
                                message = "Configured MCP Servers:\n".to_string();
                                for s in &settings.mcp_servers {
                                    message.push_str(&format!(
                                        "- {}: {} {}\n",
                                        s.name,
                                        s.command,
                                        s.args.join(" ")
                                    ));
                                }
                            }
                        }
                        "add" => {
                            if let Some(name) = args_iter.next() {
                                if let Some(cmd) = args_iter.next() {
                                    // [v3.3.3] 감사 MEDIUM-2 수정: 정규화된 서버명 충돌 검사.
                                    // 'foo.bar'와 'foo_bar'는 둘 다 'foo_bar'로 정규화되므로,
                                    // 기존 서버 중 정규화명이 충돌하는 것이 있으면 등록 거부.
                                    let sanitized_new =
                                        crate::app::App::sanitize_tool_name_part(name);
                                    let conflict = settings.mcp_servers.iter().find(|s| {
                                        s.name != name
                                            && crate::app::App::sanitize_tool_name_part(&s.name)
                                                == sanitized_new
                                    });
                                    if let Some(existing) = conflict {
                                        message = format!(
                                            "MCP 서버 '{}' 등록 불가: 기존 서버 '{}'과 정규화명 '{}'이 충돌합니다. \
                                            먼저 `/mcp remove {}`을 실행하세요.",
                                            name, existing.name, sanitized_new, existing.name
                                        );
                                    } else {
                                        let mcp_args: Vec<String> =
                                            args_iter.map(|s| s.to_string()).collect();
                                        settings.mcp_servers.retain(|s| s.name != name);
                                        settings.mcp_servers.push(
                                            crate::domain::settings::McpServerConfig {
                                                name: name.to_string(),
                                                command: cmd.to_string(),
                                                args: mcp_args,
                                            },
                                        );
                                        message = format!(
                                            "Added MCP server '{}'. Restart smlcli to load it.",
                                            name
                                        );

                                        let settings_clone = settings.clone();
                                        let tx = self.action_tx.clone();
                                        tokio::spawn(async move {
                                            let res = crate::infra::config_store::save_config(
                                                &settings_clone,
                                            )
                                            .await
                                            .map_err(|e| e.to_string());
                                            let _ = tx
                                                .send(crate::app::event_loop::Event::Action(
                                                    crate::app::action::Action::ConfigSaveFinished(
                                                        res,
                                                    ),
                                                ))
                                                .await;
                                        });
                                    }
                                } else {
                                    message =
                                        "Usage: /mcp add <name> <command> [args...]".to_string();
                                }
                            } else {
                                message = "Usage: /mcp add <name> <command> [args...]".to_string();
                            }
                        }
                        "remove" => {
                            if let Some(name) = args_iter.next() {
                                let before = settings.mcp_servers.len();
                                settings.mcp_servers.retain(|s| s.name != name);
                                if settings.mcp_servers.len() < before {
                                    message = format!(
                                        "Removed MCP server '{}'. Restart smlcli to unload it.",
                                        name
                                    );
                                    let settings_clone = settings.clone();
                                    let tx = self.action_tx.clone();
                                    tokio::spawn(async move {
                                        let res = crate::infra::config_store::save_config(
                                            &settings_clone,
                                        )
                                        .await
                                        .map_err(|e| e.to_string());
                                        let _ = tx
                                            .send(crate::app::event_loop::Event::Action(
                                                crate::app::action::Action::ConfigSaveFinished(res),
                                            ))
                                            .await;
                                    });
                                } else {
                                    message = format!("MCP server '{}' not found.", name);
                                }
                            } else {
                                message = "Usage: /mcp remove <name>".to_string();
                            }
                        }
                        _ => {
                            message = format!("Unknown mcp command: {}", subcmd);
                        }
                    }
                } else {
                    message = "Settings not available.".to_string();
                }

                self.state
                    .domain
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: Some(message),
                        tool_calls: None,
                        tool_call_id: None,
                        pinned: false,
                    });
            }
            "/undo" => {
                if let Some(settings) = &self.state.domain.settings {
                    let cwd = std::env::current_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| ".".to_string());
                    let prefix = &settings.git_integration.commit_prefix;

                    match crate::infra::git_engine::GitEngine::undo_last(&cwd, prefix) {
                        Ok(msg) => {
                            self.state
                                .ui
                                .timeline
                                .push(crate::app::state::TimelineBlock {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    kind: crate::app::state::TimelineBlockKind::GitCommit,
                                    status: crate::app::state::BlockStatus::Done,
                                    role: None,
                                    title: "Undo Successful".to_string(),
                                    subtitle: None,
                                    body: vec![crate::app::state::BlockSection::Markdown(msg)],
                                    tool_call_id: None,
                                    depth: 0,
                                    display_mode: crate::app::state::BlockDisplayMode::Expanded,
                                    diff_summary: None,
                                    created_at_ms: std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_millis()
                                        as u64,
                                    pinned: false,
                                });
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
                                    title: "Undo Failed".to_string(),
                                    subtitle: None,
                                    body: vec![crate::app::state::BlockSection::Markdown(format!(
                                        "{}",
                                        e
                                    ))],
                                    tool_call_id: None,
                                    depth: 0,
                                    display_mode: crate::app::state::BlockDisplayMode::Expanded,
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
                    self.state.ui.timeline_scroll = 0;
                    self.state.ui.timeline_follow_tail = true;
                } else {
                    self.state
                        .domain
                        .session
                        .add_message(crate::providers::types::ChatMessage {
                            role: crate::providers::types::Role::System,
                            content: Some("Settings not available.".to_string()),
                            tool_calls: None,
                            tool_call_id: None,
                            pinned: false,
                        });
                }
            }
            // ======================================================================
            // [v3.6.0] Phase 46 Task S-3/S-4: 세션 관리 명령어
            // ======================================================================
            "/new" => {
                // 현재 세션을 종료하고 새 세션을 시작합니다.
                let workspace_root = std::env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| ".".to_string());

                match crate::infra::session_log::SessionLogger::new_workspace_session(
                    &workspace_root,
                ) {
                    Ok((logger, metadata)) => {
                        // 타임라인과 세션 상태 초기화
                        self.state.ui.timeline.clear();
                        self.state.domain.session = crate::domain::session::SessionState::new();
                        self.state.domain.session_logger = Some(logger);
                        self.state.domain.current_session_metadata = Some(metadata);
                        self.state.runtime.stream_accumulator.clear();
                        self.state.runtime.active_chat_block_idx = None;
                        self.state.runtime.auto_verify = state::AutoVerifyState::Idle;
                        self.state.ui.timeline_scroll = 0;
                        self.state.ui.timeline_follow_tail = true;

                        self.state.domain.session.add_message(
                            crate::providers::types::ChatMessage {
                                role: crate::providers::types::Role::System,
                                content: Some("✨ 새 세션이 시작되었습니다.".to_string()),
                                tool_calls: None,
                                tool_call_id: None,
                                pinned: false,
                            },
                        );
                    }
                    Err(e) => {
                        self.state.domain.session.add_message(
                            crate::providers::types::ChatMessage {
                                role: crate::providers::types::Role::System,
                                content: Some(format!("새 세션 생성 실패: {}", e)),
                                tool_calls: None,
                                tool_call_id: None,
                                pinned: false,
                            },
                        );
                    }
                }
            }
            "/resume" | "/session" => {
                // 현재 워크스페이스의 세션 목록을 조회하여 타임라인에 표시합니다.
                let workspace_root = std::env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| ".".to_string());

                match crate::infra::session_log::SessionIndex::list_for_workspace(&workspace_root) {
                    Ok(sessions) => {
                        if sessions.is_empty() {
                            self.state.domain.session.add_message(
                                crate::providers::types::ChatMessage {
                                    role: crate::providers::types::Role::System,
                                    content: Some("이 워크스페이스에 저장된 세션이 없습니다. `/new`로 새 세션을 시작하세요.".to_string()),
                                    tool_calls: None,
                                    tool_call_id: None,
                                    pinned: false,
                                },
                            );
                        } else {
                            // 세션 목록을 KeyValueTable로 렌더링
                            let current_id = self
                                .state
                                .domain
                                .current_session_metadata
                                .as_ref()
                                .map(|m| m.session_id.clone())
                                .unwrap_or_default();

                            let entries: Vec<(String, String)> = sessions
                                .iter()
                                .enumerate()
                                .map(|(i, s)| {
                                    let marker = if s.session_id == current_id {
                                        " ← 현재"
                                    } else {
                                        ""
                                    };
                                    let time_str = {
                                        let secs = s.updated_at_unix_ms / 1000;
                                        let mins_ago = (std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_secs())
                                        .saturating_sub(secs);
                                        if mins_ago < 60 {
                                            format!("{}초 전", mins_ago)
                                        } else if mins_ago < 3600 {
                                            format!("{}분 전", mins_ago / 60)
                                        } else if mins_ago < 86400 {
                                            format!("{}시간 전", mins_ago / 3600)
                                        } else {
                                            format!("{}일 전", mins_ago / 86400)
                                        }
                                    };
                                    (
                                        format!("[{}]{}", i + 1, marker),
                                        format!("{} ({})", s.title, time_str),
                                    )
                                })
                                .collect();

                            let mut block = crate::app::state::TimelineBlock::new(
                                crate::app::state::TimelineBlockKind::Help,
                                "세션 목록 (Session List)",
                            );
                            block
                                .body
                                .push(crate::app::state::BlockSection::KeyValueTable(entries));
                            block.body.push(crate::app::state::BlockSection::Markdown(
                                "💡 `/resume <번호>`로 세션을 전환합니다. 예: `/resume 1`"
                                    .to_string(),
                            ));
                            self.state.ui.timeline.push(block);
                        }
                    }
                    Err(e) => {
                        self.state.domain.session.add_message(
                            crate::providers::types::ChatMessage {
                                role: crate::providers::types::Role::System,
                                content: Some(format!("세션 목록 조회 실패: {}", e)),
                                tool_calls: None,
                                tool_call_id: None,
                                pinned: false,
                            },
                        );
                    }
                }

                // /resume <번호> 형태로 세션 전환 처리
                if parts[0] == "/resume" && parts.len() > 1 {
                    let workspace_root = std::env::current_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| ".".to_string());

                    if let Ok(sessions) =
                        crate::infra::session_log::SessionIndex::list_for_workspace(&workspace_root)
                        && let Ok(idx) = parts[1].parse::<usize>()
                        && idx >= 1
                        && idx <= sessions.len()
                    {
                        let target = &sessions[idx - 1];
                        let log_dir = dirs::home_dir()
                            .unwrap_or_default()
                            .join(".smlcli")
                            .join("sessions");
                        let log_path = log_dir.join(&target.log_filename);

                        match crate::infra::session_log::SessionLogger::from_file(log_path) {
                            Ok(logger) => {
                                // 메시지 복원
                                let (messages, _errors) = logger
                                    .restore_messages()
                                    .unwrap_or_else(|_| (Vec::new(), 0));

                                // 세션 상태 교체
                                self.state.domain.session =
                                    crate::domain::session::SessionState::new();
                                for msg in messages {
                                    self.state.domain.session.messages.push(msg);
                                }
                                self.state.domain.session_logger = Some(logger);
                                self.state.domain.current_session_metadata = Some(target.clone());

                                // 타임라인 초기화 후 복원 알림
                                self.state.ui.timeline.clear();
                                self.state.ui.timeline_scroll = 0;
                                self.state.ui.timeline_follow_tail = true;
                                self.state.runtime.stream_accumulator.clear();
                                self.state.runtime.active_chat_block_idx = None;

                                // 인덱스 touch
                                let _ = crate::infra::session_log::SessionIndex::touch(
                                    &target.session_id,
                                );

                                self.state.domain.session.add_message(
                                    crate::providers::types::ChatMessage {
                                        role: crate::providers::types::Role::System,
                                        content: Some(format!(
                                            "🔄 세션 '{}' (을)를 불러왔습니다. (메시지 {}건 복원)",
                                            target.title,
                                            self.state.domain.session.messages.len() - 1
                                        )),
                                        tool_calls: None,
                                        tool_call_id: None,
                                        pinned: false,
                                    },
                                );
                            }
                            Err(e) => {
                                self.state.domain.session.add_message(
                                    crate::providers::types::ChatMessage {
                                        role: crate::providers::types::Role::System,
                                        content: Some(format!("세션 복원 실패: {}", e)),
                                        tool_calls: None,
                                        tool_call_id: None,
                                        pinned: false,
                                    },
                                );
                            }
                        }
                    }
                }
            }
            _ => {
                self.state
                    .domain
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: Some(format!("Unknown command: {}", parts[0])),
                        tool_calls: None,
                        tool_call_id: None,
                        pinned: false,
                    });
            }
        }
    }

    /// /compact 커맨드의 비동기 압축 처리 전용 헬퍼.
    /// LLM을 통해 기존 대화 컨텍스트를 요약하여 토큰을 절약.
    fn handle_compact_command(&mut self) {
        // [v0.1.0-beta.9] 중앙 보안 가드 적용: NetworkPolicy + 암호화 저장소 검증 후 압축 실행
        let (provider_kind, model_name, api_key) = match self.resolve_credentials() {
            Ok(creds) => creds,
            Err(err_msg) => {
                self.state
                    .domain
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: Some(err_msg.to_string()),
                        tool_calls: None,
                        tool_call_id: None,
                        pinned: false,
                    });
                return;
            }
        };

        let to_summarize = self.state.domain.session.extract_for_summary();
        if to_summarize.is_empty() {
            self.state
                .domain
                .session
                .add_message(crate::providers::types::ChatMessage {
                    role: crate::providers::types::Role::System,
                    content: Some("Context too small to compress.".to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                    pinned: false,
                });
        } else {
            let tx = self.action_tx.clone();

            tokio::spawn(async move {
                let mut content = "Summarize the following chat context into a brief 3-bullet list to preserve the goals and actions:\n".to_string();
                for m in to_summarize {
                    let r = match m.role {
                        crate::providers::types::Role::User => "User",
                        _ => "Other",
                    };
                    content.push_str(&format!(
                        "{}: {}\n\n",
                        r,
                        m.content.as_deref().unwrap_or_default()
                    ));
                }

                let req = crate::providers::types::ChatRequest {
                    model: model_name,
                    messages: vec![crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::User,
                        content: Some(content),
                        tool_calls: None,
                        tool_call_id: None,
                        pinned: false,
                    }],
                    stream: false,
                    tools: None,
                    tool_choice: None,
                };

                let adapter = crate::providers::registry::get_adapter(&provider_kind);
                match adapter.chat(&api_key, req).await {
                    Ok(res) => {
                        let _ = tx
                            .send(event_loop::Event::Action(action::Action::ContextSummaryOk(
                                res.message.content.unwrap_or_default(),
                            )))
                            .await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(event_loop::Event::Action(
                                action::Action::ContextSummaryErr(e.to_string()),
                            ))
                            .await;
                    }
                }
            });
        }
    }
}
