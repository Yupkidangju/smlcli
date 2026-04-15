// [v0.1.0-beta.7] Phase 3 리팩토링 2차: mod.rs에서 위자드 컨트롤러 분리.
// 설정 마법사(Setup Wizard)의 상태 전이 로직을 담당.
// ProviderSelection → ApiKeyInput → (validate) → ModelSelection → Saving 플로우를 관리.
// Config 팝업의 Enter 키 처리 로직도 이 모듈에서 담당.

use super::{App, action, event_loop, state};

impl App {
    /// 위자드의 Enter 키 처리: 각 단계별 상태 전이 및 비동기 작업 트리거.
    ///
    /// 위자드 플로우:
    /// 1. ProviderSelection: 선택된 Provider를 기록하고 ApiKeyInput으로 이동
    /// 2. ApiKeyInput: validate_credentials 비동기 호출 → (CredentialValidated 이벤트로 결과 수신)
    /// 3. ModelSelection: 선택된 모델을 기록하고 Saving으로 이동
    /// 4. Saving: 설정을 암호화 파일 및 Keyring에 저장하고 위자드 종료
    pub(crate) fn handle_wizard_enter(&mut self) {
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
                // [v0.1.0-beta.7] C-1: fetch_models 전에 반드시 validate_credentials 호출
                // OpenRouter /api/v1/models는 공개 엔드포인트라 인증 없이도 응답하므로,
                // 잘못된 키도 설정이 "성공"하던 버그를 수정.
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
                                    action::Action::CredentialValidated(Err(e.to_string())),
                                ))
                                .await;
                        }
                    }
                });
            }
            state::WizardStep::ModelSelection => {
                if !self.state.wizard.available_models.is_empty() {
                    self.state.wizard.selected_model =
                        self.state.wizard.available_models[self.state.wizard.cursor_index].clone();
                }
                self.state.wizard.step = state::WizardStep::Saving;
            }
            state::WizardStep::Saving => {
                self.save_wizard_settings();
            }
        }
    }

    /// Saving 단계: 수집된 설정을 PersistedSettings로 조립하고,
    /// Keyring에 API 키 저장, 암호화 파일에 설정 저장, AppState에 즉시 반영.
    fn save_wizard_settings(&mut self) {
        let default_model = if self.state.wizard.selected_model.is_empty() {
            "auto".to_string()
        } else {
            self.state.wizard.selected_model.clone()
        };
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

        match crate::infra::secret_store::get_or_create_master_key() {
            Ok(mk) => {
                if !self.state.wizard.api_key_input.is_empty() {
                    let key_alias = format!("{}_key", settings.default_provider.to_lowercase());
                    if let Err(e) = crate::infra::secret_store::save_api_key(
                        &key_alias,
                        &self.state.wizard.api_key_input,
                    ) {
                        self.state.wizard.err_msg =
                            Some(format!("Failed to save API key in Keyring: {}", e));
                        return;
                    }
                }
                if let Err(e) = crate::infra::config_store::save_config(&mk, &settings) {
                    self.state.wizard.err_msg = Some(format!("Failed to save settings: {}", e));
                    return;
                }
            }
            Err(e) => {
                self.state.wizard.err_msg = Some(format!("Failed to access Keyring: {}", e));
                return;
            }
        }

        self.state.settings = Some(settings); // 메모리에 반영하여 앱의 구동 상태 보장
        self.state.is_wizard_open = false;
    }

    /// Config 팝업의 Enter 키 처리: Dashboard/ProviderList/ModelList 각 화면에서의 선택 및 저장.
    pub(crate) fn handle_config_enter(&mut self) {
        match self.state.config.active_popup {
            state::ConfigPopup::Dashboard => {
                match self.state.config.cursor_index {
                    0 => {
                        // Provider 변경 진입
                        self.state.config.active_popup = state::ConfigPopup::ProviderList;
                        self.state.config.cursor_index = 0;
                    }
                    1 => {
                        // [v0.1.0-beta.10] 자체 감사: 중앙 보안 가드 적용.
                        // 이전에는 unwrap_or_default()로 빈 키를 삼키고 NetworkPolicy 검사 없이 fetch 수행.
                        let (provider_kind, _model_name, api_key) = match self.resolve_credentials()
                        {
                            Ok(creds) => creds,
                            Err(err_msg) => {
                                self.state.config.err_msg = Some(err_msg);
                                return;
                            }
                        };

                        self.state.config.active_popup = state::ConfigPopup::ModelList;
                        self.state.config.cursor_index = 0;
                        self.state.config.is_loading = true;
                        let tx = self.action_tx.clone();

                        tokio::spawn(async move {
                            let adapter = crate::providers::registry::get_adapter(&provider_kind);

                            // validate_credentials 선행 검증
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
                                        .send(event_loop::Event::Action(
                                            action::Action::ModelsFetched(
                                                Ok(models),
                                                action::FetchSource::Config,
                                            ),
                                        ))
                                        .await;
                                }
                                Err(e) => {
                                    let _ = tx
                                        .send(event_loop::Event::Action(
                                            action::Action::ModelsFetched(
                                                Err(e.to_string()),
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
                            // [v0.1.0-beta.11] 7차 감사 M-1: save_config 실패 시 err_msg 표시.
                            match crate::infra::secret_store::get_or_create_master_key() {
                                Ok(mk) => {
                                    if let Err(e) = crate::infra::config_store::save_config(&mk, s)
                                    {
                                        self.state.config.err_msg =
                                            Some(format!("설정 저장 실패: {}", e));
                                    }
                                }
                                Err(e) => {
                                    self.state.config.err_msg =
                                        Some(format!("마스터 키 접근 실패: {}", e));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            state::ConfigPopup::ProviderList => {
                let new_provider_str = if self.state.config.cursor_index == 0 {
                    "OpenRouter"
                } else {
                    "Google"
                };

                // [v0.1.0-beta.9] 중앙 보안 가드: NetworkPolicy + Keyring 사전 검증
                let (provider_kind, api_key) =
                    match self.resolve_credentials_for_provider(new_provider_str) {
                        Ok(creds) => creds,
                        Err(err_msg) => {
                            self.state.config.active_popup = state::ConfigPopup::Dashboard;
                            self.state.config.cursor_index = 0;
                            self.state.config.err_msg = Some(err_msg);
                            return;
                        }
                    };

                // [v0.1.0-beta.10] 6차 감사 H-1: 롤백 스냅샷 저장.
                // 비동기 validate_credentials/fetch_models 실패 시 복구에 사용.
                if let Some(s) = &self.state.settings {
                    self.state.config.rollback_provider = Some(s.default_provider.clone());
                    self.state.config.rollback_model = Some(s.default_model.clone());
                }

                // In-memory만 변경, 디스크 저장은 ModelList 선택 완료 시에만 수행.
                // 검증 실패 시 handle_models_fetched에서 rollback으로 복구됨.
                if let Some(s) = &mut self.state.settings {
                    s.default_provider = new_provider_str.to_string();
                    s.default_model = "auto".to_string();
                }

                // [v0.1.0-beta.9] validate_credentials → fetch_models 순서 보장.
                self.state.config.active_popup = state::ConfigPopup::ModelList;
                self.state.config.cursor_index = 0;
                self.state.config.is_loading = true;

                let tx = self.action_tx.clone();
                tokio::spawn(async move {
                    let adapter = crate::providers::registry::get_adapter(&provider_kind);

                    // 1단계: 키 유효성 검증
                    if let Err(e) = adapter.validate_credentials(&api_key).await {
                        let _ = tx
                            .send(event_loop::Event::Action(action::Action::ModelsFetched(
                                Err(format!("API key validation failed: {}", e)),
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
            state::ConfigPopup::ModelList => {
                // Model 선택 및 즉시 반영
                if !self.state.config.available_models.is_empty() {
                    let selected_model =
                        self.state.config.available_models[self.state.config.cursor_index].clone();
                    if let Some(s) = &mut self.state.settings {
                        s.default_model = selected_model;
                        // [v0.1.0-beta.11] 7차 감사 M-1: save_config 실패 시 err_msg 표시.
                        // 이 시점에서 비로소 provider 전환이 디스크에 영속화됨 (원자성 보장).
                        match crate::infra::secret_store::get_or_create_master_key() {
                            Ok(mk) => {
                                if let Err(e) = crate::infra::config_store::save_config(&mk, s) {
                                    self.state.config.err_msg =
                                        Some(format!("설정 저장 실패: {}", e));
                                }
                            }
                            Err(e) => {
                                self.state.config.err_msg =
                                    Some(format!("마스터 키 접근 실패: {}", e));
                            }
                        }
                        // 저장 성공 시 롤백 스냅샷 해제
                        if self.state.config.err_msg.is_none() {
                            self.state.config.rollback_provider = None;
                            self.state.config.rollback_model = None;
                        }
                    }
                }
                self.state.config.active_popup = state::ConfigPopup::Dashboard;
                self.state.config.cursor_index = 0;
            }
        }
    }
}
