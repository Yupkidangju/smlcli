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
                self.state.ui.config.is_open = true;
                self.state.ui.config.active_popup = state::ConfigPopup::ProviderList;
                self.state.ui.config.cursor_index = 0;
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
                let info = if let Some(s) = &self.state.domain.settings {
                    format!(
                        "Provider: {}\nModel: {}\nBudget Used: {} tokens",
                        s.default_provider,
                        s.default_model,
                        self.state.domain.session.token_budget_used
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
                    ("/config".to_string(), "설정 대시보드 (Settings Dashboard)".to_string()),
                    ("/setting".to_string(), "셋업 위자드 (Setup Wizard)".to_string()),
                    ("/provider".to_string(), "공급자 전환 (Switch Provider)".to_string()),
                    ("/model".to_string(), "모델 전환 (Switch Model)".to_string()),
                    ("/status".to_string(), "세션 상태 (Session Info)".to_string()),
                    ("/mode".to_string(), "PLAN ↔ RUN 전환 (Toggle Mode)".to_string()),
                    ("/tokens".to_string(), "토큰 사용량 (Token Usage)".to_string()),
                    ("/compact".to_string(), "컨텍스트 압축 (Compress Context)".to_string()),
                    ("/theme".to_string(), "테마 전환 (Toggle Theme)".to_string()),
                    ("/clear".to_string(), "대화 초기화 (Clear Chat)".to_string()),
                    ("/help".to_string(), "도움말 (Help)".to_string()),
                    ("/quit".to_string(), "종료 (Exit)".to_string()),
                ];
                let mut block = crate::app::state::TimelineBlock::new(
                    crate::app::state::TimelineBlockKind::Help,
                    "도움말",
                );
                block.body.push(crate::app::state::BlockSection::KeyValueTable(help_entries));
                self.state.ui.timeline.push(block);
            }
            "/quit" => {
                self.state.should_quit = true;
            }
            // [v0.1.0-beta.20] /theme 명령어: Default ↔ HighContrast 실시간 전환.
            // designs.md §21.4 요구사항 반영.
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
                    tokio::spawn(async move {
                        let _ = crate::infra::config_store::save_config(&settings_clone).await;
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
