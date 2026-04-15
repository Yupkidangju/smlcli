// [v0.1.0-beta.7] Phase 3 리팩토링: mod.rs에서 채팅/프로바이더 런타임 분리.
// LLM 요청 조립, API 키 조회, Provider 어댑터 디스패치를 전담하는 모듈.
// 자연어 입력 시 메시지 가공 → Provider 호출 → 응답 수신 흐름을 캡슐화.

use super::{App, action, event_loop};

impl App {
    /// 사용자 자연어 입력을 처리하여 LLM Provider에 채팅 요청을 전송.
    /// @ 파일 참조 인라인 처리 및 keyring 기반 API 키 조회를 포함.
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
            content: final_text,
            pinned: false,
        };
        self.state.session.add_message(msg);

        // 비동기 LLM 요청 발송
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
                // [v0.1.0-beta.7] C-3: keyring 실패 시 명시적 에러 반환 (dummy_key 제거)
                match crate::infra::secret_store::get_api_key(&alias) {
                    Ok(key) => (provider, s.default_model.clone(), key),
                    Err(_) => {
                        let _ = tx.send(event_loop::Event::Action(action::Action::ChatResponseErr(
                            "[Keyring Error] API 키를 불러올 수 없습니다. /setting으로 재설정하세요.".to_string()
                        ))).await;
                        return;
                    }
                }
            } else {
                let _ = tx
                    .send(event_loop::Event::Action(action::Action::ChatResponseErr(
                        "설정이 없습니다. /setting으로 초기 설정을 진행하세요.".to_string(),
                    )))
                    .await;
                return;
            };

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
