// [v0.1.0-beta.7] Phase 3 리팩토링: mod.rs에서 채팅/프로바이더 런타임 분리.
// LLM 요청 조립, API 키 조회, Provider 어댑터 디스패치를 전담하는 모듈.
// 자연어 입력 시 메시지 가공 → Provider 호출 → 응답 수신 흐름을 캡슐화.
//
// [v0.1.0-beta.9] 5차 감사: resolve_credentials() 중앙 보안 가드를 도입하여
// /model, /compact, /provider 등 보조 경로에서도 NetworkPolicy + 암호화 저장소 검증을 일관 적용.

use super::{App, action, event_loop};

impl App {
    /// [v0.1.0-beta.25] 도구 스키마를 포함한 표준 스트리밍 요청 생성기.
    /// 초기 요청과 Auto-Verify 재전송이 동일한 도구 능력을 갖도록 공통화한다.
    pub(crate) fn build_streaming_chat_request(
        &self,
        provider_kind: &crate::domain::provider::ProviderKind,
        model_name: String,
        messages: Vec<crate::providers::types::ChatMessage>,
    ) -> crate::providers::types::ChatRequest {
        let dialect = match provider_kind {
            crate::domain::provider::ProviderKind::Google => {
                crate::domain::provider::ToolDialect::Gemini
            }
            crate::domain::provider::ProviderKind::Anthropic => {
                crate::domain::provider::ToolDialect::Anthropic
            }
            _ => crate::domain::provider::ToolDialect::OpenAICompat,
        };
        let mut schemas = crate::tools::registry::GLOBAL_REGISTRY.all_schemas(&dialect);
        let mcp_schemas = self.state.runtime.mcp_tools_cache.clone();
        for mut schema in mcp_schemas {
            crate::tools::registry::apply_dialect(&mut schema, &dialect);
            schemas.push(schema);
        }
        let messages = self.build_messages_with_repo_map(messages);
        crate::providers::types::ChatRequest {
            model: model_name,
            messages,
            stream: true,
            tools: (!schemas.is_empty()).then_some(schemas),
            tool_choice: None,
        }
    }

    fn build_messages_with_repo_map(
        &self,
        mut messages: Vec<crate::providers::types::ChatMessage>,
    ) -> Vec<crate::providers::types::ChatMessage> {
        let Some(repo_map) = self.state.runtime.repo_map.cached.clone() else {
            return messages;
        };

        messages.retain(|msg| {
            !(msg.role == crate::providers::types::Role::System
                && msg
                    .content
                    .as_deref()
                    .unwrap_or_default()
                    .starts_with("[Repo Map]"))
        });

        let repo_map_msg = crate::providers::types::ChatMessage {
            role: crate::providers::types::Role::System,
            content: Some(repo_map),
            tool_calls: None,
            tool_call_id: None,
            pinned: true,
        };

        let insert_at = if messages
            .first()
            .is_some_and(|msg| msg.role == crate::providers::types::Role::System)
        {
            1
        } else {
            0
        };
        messages.insert(insert_at, repo_map_msg);
        messages
    }

    /// [v0.1.0-beta.9] 중앙 보안 가드: 외부 네트워크 호출 전 공통 사전 검증.
    pub(crate) fn resolve_credentials(
        &self,
    ) -> Result<
        (crate::domain::provider::ProviderKind, String, String),
        crate::domain::error::ProviderError,
    > {
        let settings = self.state.domain.settings.as_ref().ok_or_else(|| {
            crate::domain::error::ProviderError::AuthenticationFailed(
                "설정이 없습니다. /setting으로 초기 설정을 진행하세요.".to_string(),
            )
        })?;

        // NetworkPolicy 검사: Deny 시 모든 외부 호출 차단
        if settings.network_policy == crate::domain::permissions::NetworkPolicy::Deny {
            return Err(crate::domain::error::ProviderError::AuthenticationFailed(
                "[Security Block] NetworkPolicy::Deny — 외부 네트워크 접근이 차단되어 있습니다."
                    .to_string(),
            ));
        }

        let provider = match settings.default_provider.as_str() {
            "OpenAI" => crate::domain::provider::ProviderKind::OpenAI,
            "Anthropic" => crate::domain::provider::ProviderKind::Anthropic,
            "xAI" => crate::domain::provider::ProviderKind::Xai,
            "Google" => crate::domain::provider::ProviderKind::Google,
            _ => crate::domain::provider::ProviderKind::OpenRouter,
        };

        let alias = format!("{}_key", settings.default_provider.to_lowercase());
        // [v0.1.0-beta.14] 파일 기반 암호화 저장소에서 API 키 조회
        let api_key = crate::infra::secret_store::get_api_key(settings, &alias).map_err(|e| {
            crate::domain::error::ProviderError::AuthenticationFailed(format!(
                "[Config Error] API 키를 불러올 수 없습니다: {}. /setting으로 재설정하세요.",
                e
            ))
        })?;

        // secrecy::SecretString을 String으로 변환 (어댑터 호환을 위해)
        use secrecy::ExposeSecret;
        let api_key_str = api_key.expose_secret().to_string();

        if api_key_str.is_empty() {
            return Err(crate::domain::error::ProviderError::AuthenticationFailed(
                format!(
                    "[Config Error] {} API 키가 비어있습니다. /setting으로 재설정하세요.",
                    settings.default_provider
                ),
            ));
        }

        Ok((provider, settings.default_model.clone(), api_key_str))
    }

    pub(crate) fn resolve_credentials_for_provider(
        &self,
        provider_str: &str,
    ) -> Result<(crate::domain::provider::ProviderKind, String), crate::domain::error::ProviderError>
    {
        let settings = self.state.domain.settings.as_ref().ok_or_else(|| {
            crate::domain::error::ProviderError::AuthenticationFailed(
                "설정이 없습니다. /setting으로 초기 설정을 진행하세요.".to_string(),
            )
        })?;

        if settings.network_policy == crate::domain::permissions::NetworkPolicy::Deny {
            return Err(crate::domain::error::ProviderError::AuthenticationFailed(
                "[Security Block] NetworkPolicy::Deny — 외부 네트워크 접근이 차단되어 있습니다."
                    .to_string(),
            ));
        }

        let (provider, alias, needs_key) = if provider_str.starts_with("Custom: ") {
            let id = provider_str.trim_start_matches("Custom: ").to_string();
            let mut needs_k = true;
            if let Some(cp) = settings.custom_providers.iter().find(|c| c.id == id)
                && cp.auth_type.to_lowercase() == "none"
            {
                needs_k = false;
            }
            (
                crate::domain::provider::ProviderKind::Custom(id.clone()),
                format!("{}_key", id.to_lowercase()),
                needs_k,
            )
        } else {
            let p = match provider_str {
                "OpenAI" => crate::domain::provider::ProviderKind::OpenAI,
                "Anthropic" => crate::domain::provider::ProviderKind::Anthropic,
                "xAI" => crate::domain::provider::ProviderKind::Xai,
                "Google" => crate::domain::provider::ProviderKind::Google,
                _ => crate::domain::provider::ProviderKind::OpenRouter,
            };
            (p, format!("{}_key", provider_str.to_lowercase()), true)
        };

        if !needs_key {
            return Ok((provider, "".to_string()));
        }

        // [v0.1.0-beta.14] 파일 기반 암호화 저장소에서 API 키 조회
        let api_key = crate::infra::secret_store::get_api_key(settings, &alias).map_err(|e| {
            crate::domain::error::ProviderError::AuthenticationFailed(format!(
                "[Config Error] {} API 키를 불러올 수 없습니다: {}. /setting으로 재설정하세요.",
                provider_str, e
            ))
        })?;

        use secrecy::ExposeSecret;
        let api_key_str = api_key.expose_secret().to_string();

        if api_key_str.is_empty() {
            return Err(crate::domain::error::ProviderError::AuthenticationFailed(
                format!(
                    "[Config Error] {} API 키가 비어있습니다. /setting으로 재설정하세요.",
                    provider_str
                ),
            ));
        }

        Ok((provider, api_key_str))
    }

    /// 사용자 자연어 입력을 처리하여 비동기로 파일 멘션을 파싱한 뒤 LLM Provider에 전송.
    pub(crate) fn dispatch_chat_request(&mut self, text: String) {
        let tx = self.action_tx.clone();

        let mut logs = String::new();
        if text.contains("@terminal") {
            logs = self
                .state
                .runtime
                .logs_buffer
                .iter()
                .rev()
                .take(20)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");
        }

        tokio::spawn(async move {
            let mut final_text = text.clone();
            if text.contains('@') {
                let parts: Vec<&str> = text.split_whitespace().collect();
                for word in parts {
                    if word.starts_with('@') && word.len() > 1 {
                        let path = &word[1..];
                        if path == "workspace" {
                            if let Ok(mut entries) = tokio::fs::read_dir(".").await {
                                let mut dirs = vec![];
                                while let Ok(Some(e)) = entries.next_entry().await {
                                    dirs.push(e.file_name().to_string_lossy().into_owned());
                                }
                                let summary = dirs.join("\n");
                                final_text = final_text.replace(
                                    word,
                                    &format!(
                                        "\n--- Workspace Summary ---\n{}\n-------------------------\n",
                                        summary
                                    ),
                                );
                            }
                        } else if path == "terminal" {
                            final_text = final_text.replace(word, &format!("\n--- Recent Terminal Logs ---\n{}\n----------------------------\n", logs));
                        } else {
                            match tokio::fs::read_to_string(path).await {
                                Ok(content) => {
                                    final_text = final_text.replace(
                                        word,
                                        &format!(
                                            "\n--- {} ---\n{}\n--- End of {} ---\n",
                                            path, content, path
                                        ),
                                    );
                                }
                                Err(e) => {
                                    let _ = tx
                                        .send(crate::app::event_loop::Event::Action(
                                            crate::app::action::Action::AddTimelineNotice(format!(
                                                "⚠ 파일 멘션 오류 ({}): {}",
                                                path, e
                                            )),
                                        ))
                                        .await;
                                }
                            }
                        }
                    }
                }
            }
            let _ = tx
                .send(crate::app::event_loop::Event::Action(
                    crate::app::action::Action::SubmitChatRequest(final_text),
                ))
                .await;
        });
    }

    /// 파싱이 완료된 최종 텍스트를 세션에 추가하고 LLM 요청을 전송.
    pub(crate) fn submit_chat_request(&mut self, final_text: String) {
        // 사용자 메시지를 세션에 추가
        let msg = crate::providers::types::ChatMessage {
            role: crate::providers::types::Role::User,
            content: Some(final_text.clone()),
            tool_calls: None,
            tool_call_id: None,
            pinned: false,
        };
        self.state.domain.session.add_message(msg.clone());

        // [v0.1.0-beta.25] 사용자 입력 의도 분류.
        // 이제 모델의 구조화된 도구 판단을 런타임에서 차단하지 않고,
        // 설명/로깅 보조 신호로만 유지한다.
        self.state.runtime.user_intent_actionable = is_actionable_input(&final_text);
        self.refresh_repo_map_if_needed(false);
        // [v0.1.0-beta.25] 새 사용자 턴은 이전 자가 복구 세션을 종료한다.
        self.state.runtime.auto_verify = crate::app::state::AutoVerifyState::Idle;

        // [v0.1.0-beta.20] 사용자 메시지를 JSONL 세션 로그에 동기 기록
        // [v0.1.0-beta.18→20 수정] 비동기 append_message를 동기 API로 교체.
        // 이전에는 async fn의 Future를 await/spawn 없이 버려서 로그가 실행되지 않았음.
        if let Some(ref logger) = self.state.domain.session_logger
            && let Err(e) = logger.append_message(&msg)
        {
            self.state
                .runtime
                .logs_buffer
                .push(format!("[SessionLog] 사용자 메시지 기록 실패: {}", e));
        }

        // [v3.6.0] Phase 46 Task S-2: Auto-Titling 파이프라인.
        // 첫 번째 사용자 메시지일 때 프롬프트의 앞 50자를 임시 제목으로 설정.
        // 세션 인덱스를 갱신하여 /resume 시 제목이 표시되도록 함.
        if let Some(ref mut meta) = self.state.domain.current_session_metadata {
            if meta.title == "새 세션" {
                // 첫 프롬프트에서 제목 추출 (50자 제한, 줄바꿈 제거)
                let title_candidate = final_text
                    .lines()
                    .next()
                    .unwrap_or(&final_text)
                    .chars()
                    .take(50)
                    .collect::<String>();
                let title = if title_candidate.len() < final_text.len() {
                    format!("{}…", title_candidate.trim())
                } else {
                    title_candidate.trim().to_string()
                };
                meta.title = title.clone();
                meta.updated_at_unix_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let _ = crate::infra::session_log::SessionIndex::upsert(meta);
            } else {
                // 기존 세션: updated_at만 갱신
                let _ = crate::infra::session_log::SessionIndex::touch(&meta.session_id);
            }
        }

        // [v0.1.0-beta.24] Phase 15: 사용자 메시지를 블록으로 생성
        let title = final_text
            .lines()
            .next()
            .unwrap_or("User Input")
            .to_string();
        let mut block = crate::app::state::TimelineBlock::new(
            crate::app::state::TimelineBlockKind::Conversation,
            title,
        )
        .with_role(crate::providers::types::Role::User);
        block.body.push(crate::app::state::BlockSection::Markdown(
            final_text.clone(),
        ));
        self.state.ui.timeline.push(block);

        // [v0.1.0-beta.22] PLAN/RUN 모드별 시스템 프롬프트 주입 — dedupe 방식.
        // 이전 모드 지시 메시지를 찾아 교체하여 장기 세션에서 누적되지 않도록 한다.
        // "[Mode:" 접두사로 기존 모드 메시지를 식별한다.
        {
            use crate::domain::session::AppMode;
            let mode_instruction = match self.state.domain.session.mode {
                AppMode::Plan => {
                    // [v3.7.0] Phase 47 Task Q-1: AskClarification 하네싱 지침 추가.
                    // PLAN 모드에서 AI가 모호한 요구사항을 발견하면 텍스트로 질문하지 않고
                    // 반드시 AskClarification 도구를 호출하여 구조화된 선택지를 제시하도록 강제.
                    "[Mode: PLAN] You are in PLAN mode. \
                     Focus on analysis, explanation, and planning. \
                     Show code inline but do NOT automatically write files. \
                     If the user needs files created or modified, explain what you would do \
                     and ask the user to switch to RUN mode or confirm.\n\
                     IMPORTANT: If you have ANY ambiguity or need clarification from the user, \
                     DO NOT ask questions in plain text. Instead, you MUST use the 'AskClarification' tool \
                     to present structured questions with options. Each question should have a unique id, \
                     a clear title, and relevant options (or empty options for free-text input). \
                     This ensures the user can quickly select answers via the TUI form."
                }
                AppMode::Run => {
                    "[Mode: RUN] You are in RUN mode. \
                     When the user asks you to create, modify, or fix code: \
                     ALWAYS use WriteFile or ReplaceFileContent tools to make actual file changes. \
                     Do NOT just show code inline — write it to disk. \
                     For new files use WriteFile. For edits to existing files use ReplaceFileContent. \
                     Always explain what you are about to do before the tool call."
                }
            };

            // 기존 모드 메시지 교체 (dedupe)
            let mut replaced = false;
            for msg in &mut self.state.domain.session.messages {
                if msg.role == crate::providers::types::Role::System
                    && msg
                        .content
                        .as_deref()
                        .unwrap_or_default()
                        .starts_with("[Mode:")
                {
                    msg.content = Some(mode_instruction.to_string());
                    replaced = true;
                    break;
                }
            }
            // 첫 주입이면 새로 추가
            if !replaced {
                self.state
                    .domain
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: Some(mode_instruction.to_string()),
                        tool_calls: None,
                        tool_call_id: None,
                        pinned: false,
                    });
            }
        }

        // [v0.1.0-beta.9] 중앙 보안 가드 사용: dispatch 전 사전 검증
        let (provider_kind, model_name, api_key) = match self.resolve_credentials() {
            Ok(creds) => creds,
            Err(err_msg) => {
                let err_msg_str = err_msg.to_string();
                self.state
                    .domain
                    .session
                    .add_message(crate::providers::types::ChatMessage {
                        role: crate::providers::types::Role::System,
                        content: Some(err_msg_str.clone()),
                        tool_calls: None,
                        tool_call_id: None,
                        pinned: false,
                    });
                // [v0.1.0-beta.18] 에러를 타임라인에도 표시
                self.state
                    .ui
                    .timeline
                    .push(crate::app::state::TimelineBlock::new(
                        crate::app::state::TimelineBlockKind::Notice,
                        err_msg_str,
                    ));
                return;
            }
        };

        self.spawn_chat_request(provider_kind, model_name, api_key);
    }

    /// [v0.1.0-beta.18] Phase 10: 도구 결과 후 LLM 자동 재전송 (Structured Tool Loop).
    pub(crate) fn send_chat_message_internal(&mut self) {
        let (provider_kind, model_name, api_key) = match self.resolve_credentials() {
            Ok(creds) => creds,
            Err(err_msg) => {
                self.state
                    .runtime
                    .logs_buffer
                    .push(format!("[Tool Loop] 자격 증명 조회 실패: {}", err_msg));
                return;
            }
        };

        self.refresh_repo_map_if_needed(false);
        self.spawn_chat_request(provider_kind, model_name, api_key);
    }

    fn spawn_chat_request(
        &mut self,
        provider_kind: crate::domain::provider::ProviderKind,
        model_name: String,
        api_key: String,
    ) {
        // [v0.1.0-beta.26] ChatStarted 시 새 AI 블록 생성 (마지막 블록이 Assistant가 아니면)
        self.state.runtime.is_thinking = true;
        let need_new_block = match self.state.ui.timeline.last() {
            Some(block) => block.role != Some(crate::providers::types::Role::Assistant),
            None => true,
        };

        if need_new_block {
            let mut ai_block = crate::app::state::TimelineBlock::new(
                crate::app::state::TimelineBlockKind::Conversation,
                "AI",
            )
            .with_role(crate::providers::types::Role::Assistant);
            ai_block.status = crate::app::state::BlockStatus::Running;
            ai_block
                .body
                .push(crate::app::state::BlockSection::Markdown(String::new()));
            self.state.ui.timeline.push(ai_block);
        } else if let Some(block) = self.state.ui.timeline.last_mut() {
            block.status = crate::app::state::BlockStatus::Running;
            block
                .body
                .push(crate::app::state::BlockSection::Markdown(String::new()));
        }

        let idx = self.state.ui.timeline.len().saturating_sub(1);
        self.state.runtime.active_chat_block_idx = Some(idx);

        // [v1.6.0] RepoMap의 데이터 노후화(Stale Data) 결함 해결 - LLM 요청 직전 동기 갱신
        if self.state.runtime.repo_map_dirty {
            let cwd = std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string());
            if let Ok(new_map) = crate::domain::repo_map::generate_repo_map(&cwd) {
                self.state.runtime.repo_map.cached = Some(new_map);
                self.state.runtime.repo_map.stale = false;
            }
            self.state.runtime.repo_map_dirty = false;
        }

        let tx = self.action_tx.clone();
        let req = self.build_streaming_chat_request(
            &provider_kind,
            model_name,
            self.state.domain.session.messages.clone(),
        );

        tokio::spawn(async move {
            let adapter = crate::providers::registry::get_adapter(&provider_kind);

            let max_retries = 3;
            let mut attempt = 0;
            let mut final_res = Err(crate::domain::error::ProviderError::NetworkFailure(
                "Max retries exceeded".to_string(),
            ));

            while attempt < max_retries {
                let (delta_tx, mut delta_rx) = tokio::sync::mpsc::channel::<String>(64);
                let tx_delta = tx.clone();
                let delta_forwarder = tokio::spawn(async move {
                    while let Some(delta) = delta_rx.recv().await {
                        let _ = tx_delta
                            .send(event_loop::Event::Action(action::Action::ChatDelta(delta)))
                            .await;
                    }
                });

                // Request Timeout 60s
                let req_clone = req.clone();
                let timeout_res = tokio::time::timeout(
                    std::time::Duration::from_secs(60),
                    adapter.chat_stream(&api_key, req_clone, delta_tx),
                )
                .await;

                let _ = delta_forwarder.await;

                match timeout_res {
                    Ok(Ok(res)) => {
                        final_res = Ok(res);
                        break;
                    }
                    Ok(Err(e)) => {
                        let err_str = e.to_string();
                        // 429 or 5xx check
                        if err_str.contains("429")
                            || err_str.contains("500")
                            || err_str.contains("502")
                            || err_str.contains("503")
                            || err_str.contains("504")
                            || err_str.contains("Too Many Requests")
                        {
                            attempt += 1;
                            if attempt >= max_retries {
                                final_res = Err(
                                    crate::domain::error::ProviderError::NetworkFailure(err_str),
                                );
                                break;
                            }
                            // Exponential backoff
                            let backoff = std::time::Duration::from_secs(1 << attempt);
                            tokio::time::sleep(backoff).await;
                        } else {
                            final_res =
                                Err(crate::domain::error::ProviderError::NetworkFailure(err_str));
                            break;
                        }
                    }
                    Err(_) => {
                        // Timeout
                        attempt += 1;
                        if attempt >= max_retries {
                            final_res = Err(crate::domain::error::ProviderError::NetworkFailure(
                                "Request timed out after 60 seconds".to_string(),
                            ));
                            break;
                        }
                        let backoff = std::time::Duration::from_secs(1 << attempt);
                        tokio::time::sleep(backoff).await;
                    }
                }
            }

            match final_res {
                Ok(res) => {
                    let _ = tx
                        .send(event_loop::Event::Action(action::Action::ChatResponseOk(
                            Box::new(res),
                        )))
                        .await;
                }
                Err(e) => {
                    let _ = tx
                        .send(event_loop::Event::Action(action::Action::ChatResponseErr(
                            e,
                        )))
                        .await;
                }
            }
        });
    }
}

/// [v0.1.0-beta.22] 사용자 입력이 작업 요청인지 판단하는 휴리스틱 함수.
/// - false: 인삿말, 잡담, 감사 인사 등 비작업성 입력
/// - true: 파일 조작, 코드 생성, 명령 실행 등 작업 요청
///   모호한 경우 true(허용)로 기본 — false positive(차단)보다 false negative(허용)이 안전.
pub(crate) fn is_actionable_input(text: &str) -> bool {
    let trimmed = text.trim();

    // 빈 입력은 비작업성
    if trimmed.is_empty() {
        return false;
    }

    // 작업 키워드가 포함되면 항상 작업 요청으로 판단
    // (파일/디렉토리 경로, 프로그래밍 동사, @ 파일 참조 등)
    let action_keywords = [
        // 한국어 작업 동사
        "만들",
        "생성",
        "작성",
        "수정",
        "삭제",
        "읽어",
        "열어",
        "실행",
        "빌드",
        "컴파일",
        "테스트",
        "설치",
        "검색",
        "찾아",
        "분석",
        "리팩",
        "디버",
        "배포",
        "추가",
        "제거",
        "변경",
        "복사",
        "이동",
        // 영어 작업 동사
        "create",
        "make",
        "write",
        "read",
        "open",
        "run",
        "exec",
        "build",
        "compile",
        "test",
        "install",
        "search",
        "find",
        "refactor",
        "debug",
        "deploy",
        "add",
        "remove",
        "delete",
        "modify",
        "change",
        "copy",
        "move",
        "fix",
        "update",
        "edit",
        // 파일/경로 패턴
        ".rs",
        ".py",
        ".js",
        ".ts",
        ".go",
        ".java",
        ".toml",
        ".json",
        ".yaml",
        ".yml",
        ".md",
        ".txt",
        ".sh",
        ".css",
        ".html",
        "/",
        "\\",
        "src/",
        "Cargo",
        "package",
        "npm",
        "cargo",
        // @ 파일 참조
        "@",
    ];

    let lower = trimmed.to_lowercase();
    for kw in &action_keywords {
        if lower.contains(&kw.to_lowercase()) {
            return true;
        }
    }

    // 짧은 메시지(단어 3개 이하)에서 작업 키워드가 없으면 비작업성으로 판단
    let word_count = trimmed.split_whitespace().count();
    if word_count <= 3 {
        return false;
    }

    // 길이가 충분한 메시지는 작업 가능성이 있으므로 허용
    true
}
