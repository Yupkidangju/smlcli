// [v0.1.0-beta.7] Phase 3 리팩토링: mod.rs에서 채팅/프로바이더 런타임 분리.
// LLM 요청 조립, API 키 조회, Provider 어댑터 디스패치를 전담하는 모듈.
// 자연어 입력 시 메시지 가공 → Provider 호출 → 응답 수신 흐름을 캡슐화.
//
// [v0.1.0-beta.9] 5차 감사: resolve_credentials() 중앙 보안 가드를 도입하여
// /model, /compact, /provider 등 보조 경로에서도 NetworkPolicy + 암호화 저장소 검증을 일관 적용.

use super::{App, action, event_loop};

impl App {
    /// [v0.1.0-beta.9] 중앙 보안 가드: 외부 네트워크 호출 전 공통 사전 검증.
    ///
    /// 검증 항목:
    /// 1. 설정 존재 여부
    /// 2. NetworkPolicy::Deny 차단
    /// 3. 암호화 저장소에서 API 키 조회 (빈 키 거부)
    ///
    /// 성공 시 (ProviderKind, model_name, api_key) 튜플을 반환.
    /// 실패 시 사용자 표시용 에러 메시지 문자열을 반환.
    pub(crate) fn resolve_credentials(
        &self,
    ) -> Result<(crate::domain::provider::ProviderKind, String, String), String> {
        let settings = self
            .state
            .settings
            .as_ref()
            .ok_or("설정이 없습니다. /setting으로 초기 설정을 진행하세요.")?;

        // NetworkPolicy 검사: Deny 시 모든 외부 호출 차단
        if settings.network_policy == crate::domain::permissions::NetworkPolicy::Deny {
            return Err(
                "[Security Block] NetworkPolicy::Deny — 외부 네트워크 접근이 차단되어 있습니다."
                    .to_string(),
            );
        }

        let provider = match settings.default_provider.as_str() {
            "Google" => crate::domain::provider::ProviderKind::Google,
            _ => crate::domain::provider::ProviderKind::OpenRouter,
        };

        let alias = format!("{}_key", settings.default_provider.to_lowercase());
        // [v0.1.0-beta.14] 파일 기반 암호화 저장소에서 API 키 조회
        let api_key = crate::infra::secret_store::get_api_key(settings, &alias).map_err(|e| {
            format!(
                "[Config Error] API 키를 불러올 수 없습니다: {}. /setting으로 재설정하세요.",
                e
            )
        })?;

        if api_key.is_empty() {
            return Err(format!(
                "[Config Error] {} API 키가 비어있습니다. /setting으로 재설정하세요.",
                settings.default_provider
            ));
        }

        Ok((provider, settings.default_model.clone(), api_key))
    }

    /// [v0.1.0-beta.9] 특정 provider에 대한 자격 증명 해소.
    /// /provider 전환 시처럼 현재 설정과 다른 provider의 키를 조회할 때 사용.
    pub(crate) fn resolve_credentials_for_provider(
        &self,
        provider_str: &str,
    ) -> Result<(crate::domain::provider::ProviderKind, String), String> {
        let settings = self
            .state
            .settings
            .as_ref()
            .ok_or("설정이 없습니다. /setting으로 초기 설정을 진행하세요.")?;

        if settings.network_policy == crate::domain::permissions::NetworkPolicy::Deny {
            return Err(
                "[Security Block] NetworkPolicy::Deny — 외부 네트워크 접근이 차단되어 있습니다."
                    .to_string(),
            );
        }

        let provider = match provider_str {
            "Google" => crate::domain::provider::ProviderKind::Google,
            _ => crate::domain::provider::ProviderKind::OpenRouter,
        };

        let alias = format!("{}_key", provider_str.to_lowercase());
        // [v0.1.0-beta.14] 파일 기반 암호화 저장소에서 API 키 조회
        let api_key = crate::infra::secret_store::get_api_key(settings, &alias).map_err(|e| {
            format!(
                "[Config Error] {} API 키를 불러올 수 없습니다: {}. /setting으로 재설정하세요.",
                provider_str, e
            )
        })?;

        if api_key.is_empty() {
            return Err(format!(
                "[Config Error] {} API 키가 비어있습니다. /setting으로 재설정하세요.",
                provider_str
            ));
        }

        Ok((provider, api_key))
    }

    /// 사용자 자연어 입력을 처리하여 LLM Provider에 채팅 요청을 전송.
    /// @ 파일 참조 인라인 처리 및 암호화 저장소 기반 API 키 조회를 포함.
    pub(crate) fn dispatch_chat_request(&mut self, text: String) {
        // @ 파일 참조 인라인 처리: @path 형태의 토큰을 파일 내용으로 교체
        let mut final_text = text.clone();
        if text.contains('@') {
            let parts: Vec<&str> = text.split_whitespace().collect();
            for word in parts {
                if word.starts_with('@') && word.len() > 1 {
                    let path = &word[1..];
                    if let Ok(content) = std::fs::read_to_string(path) {
                        final_text = final_text.replace(
                            word,
                            &format!("\n--- {} ---\n{}\n--- End of {} ---\n", path, content, path),
                        );
                    }
                }
            }
        }

        // 사용자 메시지를 세션에 추가
        let msg = crate::providers::types::ChatMessage {
            role: crate::providers::types::Role::User,
            content: final_text.clone(),
            pinned: false,
        };
        self.state.session.add_message(msg);

        // [v0.1.0-beta.18] 사용자 메시지를 타임라인에도 추가
        self.state.timeline.push(
            crate::app::state::TimelineEntry::now(
                crate::app::state::TimelineEntryKind::UserMessage(final_text),
            ),
        );

        // [v0.1.0-beta.9] 중앙 보안 가드 사용: dispatch 전 사전 검증
        let (provider_kind, model_name, api_key) = match self.resolve_credentials() {
            Ok(creds) => creds,
            Err(err_msg) => {
                self.state
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: err_msg.clone(),
                        pinned: false,
                    });
                // [v0.1.0-beta.18] 에러를 타임라인에도 표시
                self.state.timeline.push(
                    crate::app::state::TimelineEntry::now(
                        crate::app::state::TimelineEntryKind::SystemNotice(err_msg),
                    ),
                );
                return;
            }
        };

        // [v0.1.0-beta.18] ChatStarted 이벤트 발송: thinking indicator + 빈 Delta 엔트리 추가
        // (handle_action에서 ChatStarted가 처리됨)
        self.state.is_thinking = true;
        self.state.timeline.push(
            crate::app::state::TimelineEntry::now(
                crate::app::state::TimelineEntryKind::AssistantDelta(String::new()),
            ),
        );

        // 비동기 LLM 요청 발송
        let tx = self.action_tx.clone();
        let messages = self.state.session.messages.clone();

        tokio::spawn(async move {
            let adapter = crate::providers::registry::get_adapter(&provider_kind);
            let req = crate::providers::types::ChatRequest {
                model: model_name,
                messages,
            };
            match adapter.chat(&api_key, req).await {
                Ok(res) => {
                    let _ = tx
                        .send(event_loop::Event::Action(action::Action::ChatResponseOk(
                            res,
                        )))
                        .await;
                }
                Err(e) => {
                    let _ = tx
                        .send(event_loop::Event::Action(action::Action::ChatResponseErr(
                            e.to_string(),
                        )))
                        .await;
                }
            }
        });
    }
}
