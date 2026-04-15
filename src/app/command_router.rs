// [v0.1.0-beta.7] Phase 3 리팩토링: mod.rs에서 슬래시 커맨드 엔진 분리.
// 12개의 내부 명령어(/config, /setting, /provider, /model, /status, /mode, /clear, /compact, /tokens, /help, /quit)의
// 파싱과 실행을 전담하는 모듈.
// 이전에는 mod.rs 내 handle_slash_command 메서드에 모든 로직이 인라인되어 있었음.
//
// [v0.1.0-beta.9] 5차 감사: /model과 /compact가 중앙 보안 가드(resolve_credentials)를 우회하던 문제 수정.
// unwrap_or_default()로 빈 키를 삼키던 패턴을 제거하고, NetworkPolicy + Keyring 검증을 일관 적용.

use super::{App, action, event_loop, state};

impl App {
    /// 사용자 입력이 '/'로 시작할 때 호출되는 슬래시 커맨드 라우터.
    /// 각 커맨드에 대한 상태 변경, 비동기 작업 트리거, 메시지 추가를 수행.
    pub(crate) fn handle_slash_command(&mut self, cmd: &str) {
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
                // [v0.1.0-beta.9] 중앙 보안 가드 적용: NetworkPolicy + Keyring 검증 후 모델 페칭
                let (provider_kind, _model_name, api_key) = match self.resolve_credentials() {
                    Ok(creds) => creds,
                    Err(err_msg) => {
                        self.state
                            .session
                            .add_message(crate::providers::types::ChatMessage {
                                role: crate::providers::types::Role::System,
                                content: err_msg,
                                pinned: false,
                            });
                        return;
                    }
                };

                self.state.config.is_open = true;
                self.state.config.active_popup = state::ConfigPopup::ModelList;
                self.state.config.cursor_index = 0;
                self.state.config.is_loading = true;

                let tx = self.action_tx.clone();
                tokio::spawn(async move {
                    let adapter = crate::providers::registry::get_adapter(&provider_kind);

                    // [v0.1.0-beta.10] 6차 감사 M-1: validate_credentials 선행 검증.
                    // OpenRouter /models는 공개 엔드포인트라 가짜 키도 200 반환하므로,
                    // /auth/key로 키 유효성을 먼저 확인해야 함.
                    if let Err(e) = adapter.validate_credentials(&api_key).await {
                        let _ = tx
                            .send(event_loop::Event::Action(action::Action::ModelsFetched(
                                Err(format!("API key validation failed: {}", e)),
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
                            let _ = tx
                                .send(event_loop::Event::Action(action::Action::ModelsFetched(
                                    Err(e.to_string()),
                                    action::FetchSource::Config,
                                )))
                                .await;
                        }
                    }
                });
            }
            "/status" => {
                let info = if let Some(s) = &self.state.settings {
                    format!(
                        "Provider: {}\nModel: {}\nBudget Used: {} tokens",
                        s.default_provider, s.default_model, self.state.session.token_budget_used
                    )
                } else {
                    "Not configured.".to_string()
                };
                self.state
                    .session
                    .add_message(crate::providers::types::ChatMessage {
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
                // [v0.1.0-beta.7] pinned 메시지(시스템 프롬프트, 요약)를 보존하고 나머지만 삭제.
                self.state.session.messages.retain(|m| m.pinned);
            }
            "/compact" => {
                self.handle_compact_command();
            }
            "/tokens" => {
                let budget = self.state.session.get_context_load_percentage();
                let estimated = self.state.session.estimate_current_tokens();
                let cap = self.state.session.max_token_budget;
                self.state
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: format!(
                            "[Tokens Insight]\nEstimated tokens in context: {} / {} ({}%)",
                            estimated, cap, budget
                        ),
                        pinned: false,
                    });
            }
            "/help" => {
                let help_text = "/config: Settings Dashboard\n/setting: Setup Wizard\n/provider: Switch Provider\n/model: Switch Model\n/status: Show Session Info\n/mode: Toggle PLAN/RUN\n/tokens: Show Token Limits\n/compact: Compress Chat Context\n/clear: Clear Chat\n/help: Show this message\n/quit: Exit";
                self.state
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: help_text.to_string(),
                        pinned: false,
                    });
            }
            "/quit" => {
                self.state.should_quit = true;
            }
            _ => {
                self.state
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: format!("Unknown command: {}", parts[0]),
                        pinned: false,
                    });
            }
        }
    }

    /// /compact 커맨드의 비동기 압축 처리 전용 헬퍼.
    /// LLM을 통해 기존 대화 컨텍스트를 요약하여 토큰을 절약.
    fn handle_compact_command(&mut self) {
        // [v0.1.0-beta.9] 중앙 보안 가드 적용: NetworkPolicy + Keyring 검증 후 압축 실행
        let (provider_kind, model_name, api_key) = match self.resolve_credentials() {
            Ok(creds) => creds,
            Err(err_msg) => {
                self.state
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: err_msg,
                        pinned: false,
                    });
                return;
            }
        };

        let to_summarize = self.state.session.extract_for_summary();
        if to_summarize.is_empty() {
            self.state
                .session
                .add_message(crate::providers::types::ChatMessage {
                    role: crate::providers::types::Role::System,
                    content: "Context too small to compress.".to_string(),
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
                    Ok(res) => {
                        let _ = tx
                            .send(event_loop::Event::Action(action::Action::ContextSummaryOk(
                                res.message.content,
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
