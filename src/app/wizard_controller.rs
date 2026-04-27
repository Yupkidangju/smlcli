// [v0.1.0-beta.7] Phase 3 리팩토링 2차: mod.rs에서 위자드 컨트롤러 분리.
// 설정 마법사(Setup Wizard)의 상태 전이 로직을 담당.
// ProviderSelection → ApiKeyInput → (validate) → ModelSelection → Saving 플로우를 관리.
// Config 팝업의 Enter 키 처리 로직도 이 모듈에서 담당.

use super::{App, action, event_loop, state};

#[derive(Debug, Clone, PartialEq)]
pub enum WizardError {
    MissingRequiredField(String),
}

impl App {
    /// 위자드 필수 필드 검증 (Phase 19 Audit Remediation)
    fn validate_wizard_fields(&self) -> Result<(), WizardError> {
        if self.state.ui.wizard.step == state::WizardStep::ApiKeyInput
            && self.state.ui.wizard.api_key_input.trim().is_empty()
        {
            return Err(WizardError::MissingRequiredField(
                "API Key is required.".to_string(),
            ));
        }
        Ok(())
    }
    /// 위자드의 Enter 키 처리: 각 단계별 상태 전이 및 비동기 작업 트리거.
    ///
    /// 위자드 플로우:
    /// 1. ProviderSelection: 선택된 Provider를 기록하고 ApiKeyInput으로 이동
    /// 2. ApiKeyInput: validate_credentials 비동기 호출 → (CredentialValidated 이벤트로 결과 수신)
    /// 3. ModelSelection: 선택된 모델을 기록하고 Saving으로 이동
    /// 4. Saving: 설정을 암호화 저장소에 저장하고 위자드 종료
    pub(crate) fn handle_wizard_enter(&mut self) {
        match self.state.ui.wizard.step {
            state::WizardStep::ProviderSelection => {
                self.state.ui.wizard.selected_provider = match self.state.ui.wizard.cursor_index {
                    0 => Some(crate::domain::provider::ProviderKind::OpenAI),
                    1 => Some(crate::domain::provider::ProviderKind::Anthropic),
                    2 => Some(crate::domain::provider::ProviderKind::Xai),
                    3 => Some(crate::domain::provider::ProviderKind::OpenRouter),
                    _ => Some(crate::domain::provider::ProviderKind::Google),
                };
                self.state.ui.wizard.step = state::WizardStep::ApiKeyInput;
                self.state.ui.wizard.cursor_index = 0;
            }
            state::WizardStep::ApiKeyInput => {
                // [v1.0.0] 필수 필드 검증 누락 시 상태 유지 및 에러 표출, 버퍼 초기화 (ClearBuffer)
                if let Err(WizardError::MissingRequiredField(msg)) = self.validate_wizard_fields() {
                    self.state.ui.wizard.err_msg = Some(msg);
                    self.state.ui.wizard.api_key_input.clear();
                    return;
                }

                // [v0.1.0-beta.7] C-1: fetch_models 전에 반드시 validate_credentials 호출
                // OpenRouter /api/v1/models는 공개 엔드포인트라 인증 없이도 응답하므로,
                // 잘못된 키도 설정이 "성공"하던 버그를 수정.
                self.state.ui.wizard.is_loading_models = true;
                self.state.ui.wizard.err_msg = None;

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
                    match adapter.validate_credentials(&api_key).await {
                        Ok(()) => {
                            let _ = tx
                                .send(event_loop::Event::Action(
                                    action::Action::CredentialValidated(Ok(())),
                                ))
                                .await;
                        }
                        Err(e) => {
                            // [v0.1.0-beta.21] ProviderError 구조화
                            let _ = tx
                                .send(event_loop::Event::Action(
                                    action::Action::CredentialValidated(Err(
                                        crate::domain::error::ProviderError::AuthenticationFailed(
                                            e.to_string(),
                                        ),
                                    )),
                                ))
                                .await;
                        }
                    }
                });
            }
            state::WizardStep::ModelSelection => {
                if !self.state.ui.wizard.available_models.is_empty() {
                    self.state.ui.wizard.selected_model = self.state.ui.wizard.available_models
                        [self.state.ui.wizard.cursor_index]
                        .clone();
                }
                self.state.ui.wizard.step = state::WizardStep::Saving;
            }
            state::WizardStep::Saving => {
                if !self.state.ui.wizard.is_loading_models {
                    self.save_wizard_settings();
                }
            }
        }
    }

    /// Saving 단계: 수집된 설정을 PersistedSettings로 조립하고,
    /// 암호화 저장소에 API 키 저장, config.toml에 설정 저장, AppState에 즉시 반영.
    fn save_wizard_settings(&mut self) {
        let default_model = if self.state.ui.wizard.selected_model.is_empty() {
            "auto".to_string()
        } else {
            self.state.ui.wizard.selected_model.clone()
        };
        let provider_str = match &self.state.ui.wizard.selected_provider {
            Some(crate::domain::provider::ProviderKind::OpenAI) => "OpenAI".to_string(),
            Some(crate::domain::provider::ProviderKind::Anthropic) => "Anthropic".to_string(),
            Some(crate::domain::provider::ProviderKind::Xai) => "xAI".to_string(),
            Some(crate::domain::provider::ProviderKind::Google) => "Google".to_string(),
            _ => "OpenRouter".to_string(),
        };
        // [v0.1.0-beta.14] encrypted_keys 필드 추가, keyring 제거
        let mut settings = crate::domain::settings::PersistedSettings {
            version: 1,
            default_provider: provider_str,
            default_model,
            shell_policy: crate::domain::permissions::ShellPolicy::Ask,
            file_write_policy: crate::domain::permissions::FileWritePolicy::AlwaysAsk,
            // [v2.5.0] Safe Starter preset (designs.md §8 Step 4): network AllowAll
            network_policy: crate::domain::permissions::NetworkPolicy::AllowAll,
            safe_commands: None,
            encrypted_keys: std::collections::HashMap::new(),
            theme: "default".to_string(),
            ..Default::default()
        };

        // API 키를 암호화하여 settings.encrypted_keys에 저장
        if !self.state.ui.wizard.api_key_input.is_empty() {
            let key_alias = format!("{}_key", settings.default_provider.to_lowercase());
            use secrecy::SecretString;
            let secret = SecretString::new(self.state.ui.wizard.api_key_input.clone().into());
            if let Err(e) =
                crate::infra::secret_store::save_api_key(&mut settings, &key_alias, &secret)
            {
                self.state.ui.wizard.err_msg = Some(format!("API 키 암호화 실패: {}", e));
                return;
            }
        }

        // 설정을 TOML로 디스크에 비동기 저장
        let tx = self.action_tx.clone();
        let settings_clone = settings.clone();

        tokio::spawn(async move {
            match crate::infra::config_store::save_config(&settings_clone).await {
                Ok(_) => {
                    let _ = tx
                        .send(event_loop::Event::Action(
                            action::Action::WizardSaveFinished(Ok(())),
                        ))
                        .await;
                }
                Err(e) => {
                    let _ = tx
                        .send(event_loop::Event::Action(
                            action::Action::WizardSaveFinished(Err(e.to_string())),
                        ))
                        .await;
                }
            }
        });

        self.state.domain.settings = Some(settings); // 메모리에 반영하여 앱의 구동 상태 보장
        self.state.ui.wizard.is_loading_models = true; // 저장 중 스피너 표시 등 로딩 상태 활용
    }

    /// Config 팝업의 Enter 키 처리: Dashboard/ProviderList/ModelList 각 화면에서의 선택 및 저장.
    pub(crate) fn handle_config_enter(&mut self) {
        match self.state.ui.config.active_popup {
            state::ConfigPopup::Dashboard => {
                match self.state.ui.config.cursor_index {
                    0 => {
                        // Provider 변경 진입
                        self.state.ui.config.active_popup = state::ConfigPopup::ProviderList;
                        self.state.ui.config.cursor_index = 0;
                    }
                    1 => {
                        // [v0.1.0-beta.10] 자체 감사: 중앙 보안 가드 적용.
                        // 이전에는 unwrap_or_default()로 빈 키를 삼키고 NetworkPolicy 검사 없이 fetch 수행.
                        let (provider_kind, _model_name, api_key) = match self.resolve_credentials()
                        {
                            Ok(creds) => creds,
                            Err(err_msg) => {
                                self.state.ui.config.err_msg = Some(err_msg.to_string());
                                return;
                            }
                        };

                        self.state.ui.config.active_popup = state::ConfigPopup::ModelList;
                        self.state.ui.config.cursor_index = 0;
                        self.state.ui.config.is_loading = true;
                        let tx = self.action_tx.clone();

                        tokio::spawn(async move {
                            let adapter = crate::providers::registry::get_adapter(&provider_kind);

                            // validate_credentials 선행 검증
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
                                        .send(event_loop::Event::Action(
                                            action::Action::ModelsFetched(
                                                Ok(models),
                                                action::FetchSource::Config,
                                            ),
                                        ))
                                        .await;
                                }
                                Err(e) => {
                                    // [v0.1.0-beta.21] ProviderError 구조화
                                    let _ = tx
                                        .send(event_loop::Event::Action(
                                            action::Action::ModelsFetched(
                                                Err(crate::domain::error::ProviderError::NetworkFailure(
                                                    e.to_string(),
                                                )),
                                                action::FetchSource::Config,
                                            ),
                                        ))
                                        .await;
                                }
                            }
                        });
                    }
                    2 => {
                        // ShellPolicy 토글: Ask → SafeOnly → Deny → Ask
                        if let Some(s) = &mut self.state.domain.settings {
                            let _old_policy = s.shell_policy.clone();
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
                            let settings_clone = s.clone();
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
                    }
                    3 => {
                        // NetworkPolicy 토글: ProviderOnly -> AllowAll -> Deny -> ProviderOnly
                        if let Some(s) = &mut self.state.domain.settings {
                            let _old_policy = s.network_policy.clone();
                            s.network_policy = match s.network_policy {
                                crate::domain::permissions::NetworkPolicy::ProviderOnly => {
                                    crate::domain::permissions::NetworkPolicy::AllowAll
                                }
                                crate::domain::permissions::NetworkPolicy::AllowAll => {
                                    crate::domain::permissions::NetworkPolicy::Deny
                                }
                                crate::domain::permissions::NetworkPolicy::Deny => {
                                    crate::domain::permissions::NetworkPolicy::ProviderOnly
                                }
                            };
                            let settings_clone = s.clone();
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
                    }
                    4 => {
                        // Sandbox 토글
                        if let Some(s) = &mut self.state.domain.settings {
                            s.sandbox.enabled = !s.sandbox.enabled;
                            let settings_clone = s.clone();
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
                    }
                    _ => {}
                }
            }
            state::ConfigPopup::ProviderList => {
                let new_provider_str = match self.state.ui.config.cursor_index {
                    0 => "OpenAI".to_string(),
                    1 => "Anthropic".to_string(),
                    2 => "xAI".to_string(),
                    3 => "OpenRouter".to_string(),
                    4 => "Google".to_string(),
                    idx => {
                        let mut res = "OpenAI".to_string();
                        if let Some(settings) = &self.state.domain.settings
                            && let Some(cp) = settings.custom_providers.get(idx.saturating_sub(5))
                        {
                            res = format!("Custom: {}", cp.id);
                        }
                        res
                    }
                };

                // [v0.1.0-beta.9] 중앙 보안 가드: NetworkPolicy + 암호화 저장소 사전 검증
                let (provider_kind, api_key) =
                    match self.resolve_credentials_for_provider(&new_provider_str) {
                        Ok(creds) => creds,
                        Err(err_msg) => {
                            self.state.ui.config.active_popup = state::ConfigPopup::Dashboard;
                            self.state.ui.config.cursor_index = 0;
                            self.state.ui.config.err_msg = Some(err_msg.to_string());
                            return;
                        }
                    };

                // [v0.1.0-beta.10] 6차 감사 H-1: 롤백 스냅샷 저장.
                // 비동기 validate_credentials/fetch_models 실패 시 복구에 사용.
                if let Some(s) = &self.state.domain.settings {
                    self.state.ui.config.rollback_provider = Some(s.default_provider.clone());
                    self.state.ui.config.rollback_model = Some(s.default_model.clone());
                }

                // In-memory만 변경, 디스크 저장은 ModelList 선택 완료 시에만 수행.
                // 검증 실패 시 handle_models_fetched에서 rollback으로 복구됨.
                if let Some(s) = &mut self.state.domain.settings {
                    s.default_provider = new_provider_str.to_string();
                    s.default_model = "auto".to_string();
                }

                // [v0.1.0-beta.9] validate_credentials → fetch_models 순서 보장.
                self.state.ui.config.active_popup = state::ConfigPopup::ModelList;
                self.state.ui.config.cursor_index = 0;
                self.state.ui.config.is_loading = true;

                let tx = self.action_tx.clone();
                tokio::spawn(async move {
                    let adapter = crate::providers::registry::get_adapter(&provider_kind);

                    // 1단계: 키 유효성 검증
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

                    // 2단계: 모델 목록 조회
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
            state::ConfigPopup::ModelList => {
                // Model 선택 및 즉시 반영
                if !self.state.ui.config.available_models.is_empty() {
                    let selected_model = self.state.ui.config.available_models
                        [self.state.ui.config.cursor_index]
                        .clone();
                    if let Some(s) = &mut self.state.domain.settings {
                        // [v0.1.0-beta.12] 8차 감사 M-1: 저장 실패 시 복구를 위해 이전 model 보존
                        let _old_model = s.default_model.clone();
                        s.default_model = selected_model;

                        // [v0.1.0-beta.14] 이 시점에서 비로소 provider 전환이 디스크에 영속화됨 (원자성 보장).
                        let settings_clone = s.clone();
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
                        crate::providers::registry::reload_providers(); // [v1.2.0] 동적 갱신

                        // 저장 성공 가정 (비동기라 에러 피드백은 향후 개선 필요)
                        self.state.ui.config.rollback_provider = None;
                        self.state.ui.config.rollback_model = None;
                    }
                }
                self.state.ui.config.active_popup = state::ConfigPopup::Dashboard;
                self.state.ui.config.cursor_index = 0;
            }
        }
    }
}
