use crate::domain::error::ProviderError;
use crate::domain::provider::ProviderKind;
use reqwest::Client;
use std::future::Future;
use std::pin::Pin;

pub trait ProviderAdapter: Send + Sync {
    /// 해당 Provider에 대한 API 인증 정보를 최소한으로 검증하는 smoke test 함수
    fn validate_credentials<'a>(
        &'a self,
        api_key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), ProviderError>> + Send + 'a>>;

    /// Provider에 맞추어 채팅 요청을 전송하고 응답을 반환
    fn chat<'a>(
        &'a self,
        api_key: &'a str,
        req: crate::providers::types::ChatRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<crate::providers::types::ChatResponse, ProviderError>>
                + Send
                + 'a,
        >,
    >;

    /// [v0.1.0-beta.18] Phase 10: SSE 스트리밍 채팅.
    /// 델타 토큰을 tx로 실시간 전송하고, 완료 시 전체 응답을 반환.
    fn chat_stream<'a>(
        &'a self,
        api_key: &'a str,
        req: crate::providers::types::ChatRequest,
        delta_tx: tokio::sync::mpsc::Sender<String>,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<crate::providers::types::ChatResponse, ProviderError>>
                + Send
                + 'a,
        >,
    >;

    /// 지원하는 모델 목록을 동적으로 가져옴
    fn fetch_models<'a>(
        &'a self,
        api_key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, ProviderError>> + Send + 'a>>;
}

/// [v2.5.2] 감사 HIGH-2: 인증 전략을 어댑터 수준에서 지원.
/// Bearer, None, CustomHeader 세 가지 방식으로 커스텀 Provider를 유연하게 인증.
#[derive(Debug, Clone)]
pub enum AuthStrategy {
    /// 표준 Bearer 토큰 인증 (기본값)
    Bearer,
    /// 인증 없음 (로컬 Ollama, LMStudio 등)
    None,
    /// 커스텀 헤더 이름으로 API 키 전달
    CustomHeader(String),
}

struct FailClosedProviderAdapter {
    reason: String,
}

impl FailClosedProviderAdapter {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }

    fn error(&self) -> ProviderError {
        ProviderError::Configuration(self.reason.clone())
    }
}

impl ProviderAdapter for FailClosedProviderAdapter {
    fn validate_credentials<'a>(
        &'a self,
        _api_key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), ProviderError>> + Send + 'a>> {
        let error = self.error();
        Box::pin(async move { Err(error) })
    }

    fn chat<'a>(
        &'a self,
        _api_key: &'a str,
        _req: crate::providers::types::ChatRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<crate::providers::types::ChatResponse, ProviderError>>
                + Send
                + 'a,
        >,
    > {
        let error = self.error();
        Box::pin(async move { Err(error) })
    }

    fn chat_stream<'a>(
        &'a self,
        _api_key: &'a str,
        _req: crate::providers::types::ChatRequest,
        _delta_tx: tokio::sync::mpsc::Sender<String>,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<crate::providers::types::ChatResponse, ProviderError>>
                + Send
                + 'a,
        >,
    > {
        let error = self.error();
        Box::pin(async move { Err(error) })
    }

    fn fetch_models<'a>(
        &'a self,
        _api_key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, ProviderError>> + Send + 'a>> {
        let error = self.error();
        Box::pin(async move { Err(error) })
    }
}

#[derive(Clone)]
pub struct OpenAICompatAdapter {
    client: Client,
    base_url: String,
    auth_strategy: AuthStrategy,
}

impl OpenAICompatAdapter {
    fn http_client() -> Client {
        Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("reqwest client with redirect disabled")
    }

    pub fn new(base_url: String) -> Self {
        Self {
            client: Self::http_client(),
            base_url,
            auth_strategy: AuthStrategy::Bearer,
        }
    }

    /// [v2.5.2] 커스텀 인증 전략으로 어댑터 생성
    pub fn with_auth(base_url: String, auth_strategy: AuthStrategy) -> Self {
        Self {
            client: Self::http_client(),
            base_url,
            auth_strategy,
        }
    }

    /// [v2.5.2] 인증 전략에 따른 요청 빌더 헤더 설정 헬퍼
    fn apply_auth(
        &self,
        builder: reqwest::RequestBuilder,
        api_key: &str,
    ) -> reqwest::RequestBuilder {
        match &self.auth_strategy {
            AuthStrategy::Bearer => builder.header("Authorization", format!("Bearer {}", api_key)),
            AuthStrategy::None => builder, // 헤더 없이 전송
            AuthStrategy::CustomHeader(name) => builder.header(name, api_key),
        }
    }
}

impl ProviderAdapter for OpenAICompatAdapter {
    fn validate_credentials<'a>(
        &'a self,
        api_key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), ProviderError>> + Send + 'a>> {
        Box::pin(async move {
            let url = if self.base_url.contains("openrouter.ai") {
                "https://openrouter.ai/api/v1/auth/key".to_string()
            } else {
                format!("{}/models", self.base_url)
            };

            let response = self
                .apply_auth(self.client.get(&url), api_key)
                .send()
                .await
                .map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;

            if response.status().is_success() {
                Ok(())
            } else {
                Err(ProviderError::AuthenticationFailed(
                    "Invalid API Key".into(),
                ))
            }
        })
    }

    fn chat<'a>(
        &'a self,
        api_key: &'a str,
        req: crate::providers::types::ChatRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<crate::providers::types::ChatResponse, ProviderError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            #[derive(serde::Serialize)]
            struct Payload<'a> {
                model: &'a str,
                messages: &'a Vec<crate::providers::types::ChatMessage>,
                #[serde(skip_serializing_if = "Option::is_none")]
                tools: &'a Option<Vec<serde_json::Value>>,
                #[serde(skip_serializing_if = "Option::is_none")]
                tool_choice: &'a Option<String>,
            }
            // [v1.8.0] Phase 26: 빈 응답이나 호환되지 않는 메시지 정제
            let mut sanitized_messages = Vec::new();
            for msg in &req.messages {
                let mut sanitized_msg = msg.clone();
                if sanitized_msg.role == crate::providers::types::Role::Assistant {
                    let has_tools = sanitized_msg
                        .tool_calls
                        .as_ref()
                        .is_some_and(|t| !t.is_empty());
                    let is_empty = sanitized_msg
                        .content
                        .as_ref()
                        .is_none_or(|c| c.trim().is_empty());
                    if is_empty && !has_tools {
                        continue;
                    }
                    if has_tools && is_empty {
                        sanitized_msg.content = None;
                    }
                } else if sanitized_msg.role == crate::providers::types::Role::Tool
                    && sanitized_msg
                        .content
                        .as_ref()
                        .is_none_or(|c| c.trim().is_empty())
                {
                    sanitized_msg.content = Some("Success".to_string());
                }
                sanitized_messages.push(sanitized_msg);
            }

            let payload = Payload {
                model: &req.model,
                messages: &sanitized_messages,
                tools: &req.tools,
                tool_choice: &req.tool_choice,
            };

            let response = self
                .apply_auth(
                    self.client
                        .post(format!("{}/chat/completions", self.base_url)),
                    api_key,
                )
                .json(&payload)
                .send()
                .await
                .map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;

            if !response.status().is_success() {
                let code = response.status().as_u16();
                let err_text =
                    crate::providers::streaming::bounded_error_body(response, &[api_key]).await;
                return Err(ProviderError::ApiResponse {
                    code,
                    message: format!("API Error: {}", err_text),
                });
            }

            #[derive(serde::Deserialize)]
            struct OpenRouterRes {
                choices: Vec<Choice>,
            }
            #[derive(serde::Deserialize)]
            struct Choice {
                message: Message,
            }
            #[derive(serde::Deserialize)]
            struct Message {
                content: String,
            }

            let mut parsed: OpenRouterRes =
                crate::providers::streaming::bounded_json_response(response).await?;
            let reply_content = if !parsed.choices.is_empty() {
                parsed.choices.remove(0).message.content
            } else {
                "No response from model.".to_string()
            };

            let reply = crate::providers::types::ChatMessage {
                role: crate::providers::types::Role::Assistant,
                content: Some(reply_content),
                tool_calls: None,
                tool_call_id: None,
                pinned: false,
            };

            Ok(crate::providers::types::ChatResponse {
                message: reply,
                input_tokens: 0,
                output_tokens: 0,
            })
        })
    }

    // [v0.1.0-beta.18] Phase 10: OpenRouter SSE 스트리밍.
    // stream: true 파라미터 전송 → data: ... SSE 이벤트 수신 → delta 토큰 추출 → tx로 전송.
    fn chat_stream<'a>(
        &'a self,
        api_key: &'a str,
        req: crate::providers::types::ChatRequest,
        delta_tx: tokio::sync::mpsc::Sender<String>,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<crate::providers::types::ChatResponse, ProviderError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            #[derive(serde::Serialize)]
            struct StreamPayload<'a> {
                model: &'a str,
                messages: &'a Vec<crate::providers::types::ChatMessage>,
                stream: bool,
                #[serde(skip_serializing_if = "Option::is_none")]
                tools: &'a Option<Vec<serde_json::Value>>,
                #[serde(skip_serializing_if = "Option::is_none")]
                tool_choice: &'a Option<String>,
            }
            // [v1.8.0] Phase 26: 빈 응답이나 호환되지 않는 메시지 정제
            let mut sanitized_messages = Vec::new();
            for msg in &req.messages {
                let mut sanitized_msg = msg.clone();
                if sanitized_msg.role == crate::providers::types::Role::Assistant {
                    let has_tools = sanitized_msg
                        .tool_calls
                        .as_ref()
                        .is_some_and(|t| !t.is_empty());
                    let is_empty = sanitized_msg
                        .content
                        .as_ref()
                        .is_none_or(|c| c.trim().is_empty());
                    if is_empty && !has_tools {
                        continue;
                    }
                    if has_tools && is_empty {
                        sanitized_msg.content = None;
                    }
                } else if sanitized_msg.role == crate::providers::types::Role::Tool
                    && sanitized_msg
                        .content
                        .as_ref()
                        .is_none_or(|c| c.trim().is_empty())
                {
                    sanitized_msg.content = Some("Success".to_string());
                }
                sanitized_messages.push(sanitized_msg);
            }

            let payload = StreamPayload {
                model: &req.model,
                messages: &sanitized_messages,
                stream: true,
                tools: &req.tools,
                tool_choice: &req.tool_choice,
            };

            let response = self
                .apply_auth(
                    self.client
                        .post(format!("{}/chat/completions", self.base_url)),
                    api_key,
                )
                .json(&payload)
                .send()
                .await
                .map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;

            if !response.status().is_success() {
                let code = response.status().as_u16();
                let err_text =
                    crate::providers::streaming::bounded_error_body(response, &[api_key]).await;
                return Err(ProviderError::ApiResponse {
                    code,
                    message: format!("Stream API Error: {}", err_text),
                });
            }

            let (full_content, tool_calls) =
                crate::providers::streaming::openai_compatible_stream(response, delta_tx).await?;

            let reply = crate::providers::types::ChatMessage {
                role: crate::providers::types::Role::Assistant,
                content: if full_content.is_empty() {
                    None
                } else {
                    Some(full_content)
                },
                tool_calls,
                tool_call_id: None,
                pinned: false,
            };

            Ok(crate::providers::types::ChatResponse {
                message: reply,
                input_tokens: 0,
                output_tokens: 0,
            })
        })
    }

    fn fetch_models<'a>(
        &'a self,
        api_key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, ProviderError>> + Send + 'a>> {
        Box::pin(async move {
            let response = self
                .apply_auth(
                    self.client.get(format!("{}/models", self.base_url)),
                    api_key,
                )
                .send()
                .await
                .map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;
            if !response.status().is_success() {
                return Err(ProviderError::NetworkFailure(
                    "Failed to fetch models".into(),
                ));
            }
            #[derive(serde::Deserialize)]
            struct ModelObj {
                id: String,
            }
            #[derive(serde::Deserialize)]
            struct ModelRes {
                data: Vec<ModelObj>,
            }

            let parsed: ModelRes =
                crate::providers::streaming::bounded_json_response(response).await?;
            Ok(parsed.data.into_iter().map(|m| m.id).collect())
        })
    }
}

pub struct GeminiAdapter {
    client: Client,
}

impl GeminiAdapter {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl ProviderAdapter for GeminiAdapter {
    fn validate_credentials<'a>(
        &'a self,
        api_key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), ProviderError>> + Send + 'a>> {
        Box::pin(async move {
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                api_key
            );
            let response = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;

            if response.status().is_success() {
                Ok(())
            } else {
                Err(ProviderError::AuthenticationFailed(
                    "Invalid Gemini API Key".into(),
                ))
            }
        })
    }

    fn chat<'a>(
        &'a self,
        api_key: &'a str,
        req: crate::providers::types::ChatRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<crate::providers::types::ChatResponse, ProviderError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            // Gemini의 OpenAI 호환 엔드포인트를 사용하여 완벽한 구조체 호환 통신 수행
            #[derive(serde::Serialize)]
            struct Payload<'a> {
                model: &'a str,
                messages: &'a Vec<crate::providers::types::ChatMessage>,
                #[serde(skip_serializing_if = "Option::is_none")]
                tools: &'a Option<Vec<serde_json::Value>>,
                #[serde(skip_serializing_if = "Option::is_none")]
                tool_choice: &'a Option<String>,
            }
            let payload = Payload {
                model: &req.model,
                messages: &req.messages,
                tools: &req.tools,
                tool_choice: &req.tool_choice,
            };

            let response = self
                .client
                .post("https://generativelanguage.googleapis.com/v1beta/openai/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&payload)
                .send()
                .await
                .map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;

            if !response.status().is_success() {
                let code = response.status().as_u16();
                let err_text =
                    crate::providers::streaming::bounded_error_body(response, &[api_key]).await;
                return Err(ProviderError::ApiResponse {
                    code,
                    message: format!("Gemini Error: {}", err_text),
                });
            }

            #[derive(serde::Deserialize)]
            struct GeminiRes {
                choices: Vec<Choice>,
            }
            #[derive(serde::Deserialize)]
            struct Choice {
                message: Message,
            }
            #[derive(serde::Deserialize)]
            struct Message {
                content: Option<String>,
                tool_calls: Option<Vec<crate::providers::types::ToolCallRequest>>,
            }

            let mut parsed: GeminiRes =
                crate::providers::streaming::bounded_json_response(response).await?;

            let (reply_content, tool_calls) = if !parsed.choices.is_empty() {
                let msg = parsed.choices.remove(0).message;
                (msg.content, msg.tool_calls)
            } else {
                (Some("No response from Gemini API.".to_string()), None)
            };

            let reply = crate::providers::types::ChatMessage {
                role: crate::providers::types::Role::Assistant,
                content: reply_content,
                tool_calls,
                tool_call_id: None,
                pinned: false,
            };

            Ok(crate::providers::types::ChatResponse {
                message: reply,
                input_tokens: 0,
                output_tokens: 0,
            })
        })
    }

    // [v0.1.0-beta.18] Phase 10: Gemini SSE 스트리밍 (OpenAI 호환 엔드포인트).
    fn chat_stream<'a>(
        &'a self,
        api_key: &'a str,
        req: crate::providers::types::ChatRequest,
        delta_tx: tokio::sync::mpsc::Sender<String>,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<crate::providers::types::ChatResponse, ProviderError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            #[derive(serde::Serialize)]
            struct StreamPayload<'a> {
                model: &'a str,
                messages: &'a Vec<crate::providers::types::ChatMessage>,
                stream: bool,
                #[serde(skip_serializing_if = "Option::is_none")]
                tools: &'a Option<Vec<serde_json::Value>>,
                #[serde(skip_serializing_if = "Option::is_none")]
                tool_choice: &'a Option<String>,
            }
            let payload = StreamPayload {
                model: &req.model,
                messages: &req.messages,
                stream: true,
                tools: &req.tools,
                tool_choice: &req.tool_choice,
            };

            let response = self
                .client
                .post("https://generativelanguage.googleapis.com/v1beta/openai/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&payload)
                .send()
                .await
                .map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;

            if !response.status().is_success() {
                let code = response.status().as_u16();
                let err_text =
                    crate::providers::streaming::bounded_error_body(response, &[api_key]).await;
                return Err(ProviderError::ApiResponse {
                    code,
                    message: format!("Gemini Stream Error: {}", err_text),
                });
            }

            let (full_content, tool_calls) =
                crate::providers::streaming::openai_compatible_stream(response, delta_tx).await?;

            let reply = crate::providers::types::ChatMessage {
                role: crate::providers::types::Role::Assistant,
                content: if full_content.is_empty() {
                    None
                } else {
                    Some(full_content)
                },
                tool_calls,
                tool_call_id: None,
                pinned: false,
            };

            Ok(crate::providers::types::ChatResponse {
                message: reply,
                input_tokens: 0,
                output_tokens: 0,
            })
        })
    }

    fn fetch_models<'a>(
        &'a self,
        api_key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, ProviderError>> + Send + 'a>> {
        Box::pin(async move {
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                api_key
            );
            let response = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;
            if !response.status().is_success() {
                return Err(ProviderError::NetworkFailure(
                    "Failed to fetch Gemini models".into(),
                ));
            }
            #[derive(serde::Deserialize)]
            struct ModelObj {
                name: String,
            }
            #[derive(serde::Deserialize)]
            struct ModelRes {
                models: Vec<ModelObj>,
            }

            let parsed: ModelRes =
                crate::providers::streaming::bounded_json_response(response).await?;
            // [v0.1.0-beta.7] Gemini API는 name을 "models/gemini-..." 형태로 반환하지만,
            // OpenAI 호환 엔드포인트의 chat/completions는 bare model id (예: "gemini-2.0-flash")를 요구함.
            // 공식 문서(https://ai.google.dev/gemini-api/docs/openai)의 예시: model="gemini-3-flash-preview"
            // 따라서 "models/" 프리픽스를 반드시 제거해야 채팅 요청 시 정상 동작함.
            Ok(parsed
                .models
                .into_iter()
                .map(|m| {
                    m.name
                        .strip_prefix("models/")
                        .unwrap_or(&m.name)
                        .to_string()
                })
                .collect())
        })
    }
}

use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::RwLock;

// [v3.4.0] Phase 44 Task D-2: cfg(test) 한정 구조적 미사용.
// 테스트 환경에서 get_adapter()가 MockProvider를 직접 반환하므로 필드가 미사용.
// 이 allow는 테스트 리팩토링(DI 주입 방식 변경) 전까지 유지.
#[allow(dead_code)]
pub struct ProviderRegistry {
    openai: Arc<OpenAICompatAdapter>,
    xai: Arc<OpenAICompatAdapter>,
    openrouter: Arc<OpenAICompatAdapter>,
    anthropic: Arc<crate::providers::anthropic::AnthropicAdapter>,
    google: Arc<GeminiAdapter>,
    // [v3.7.2] LM Studio의 동적인 base_url 변경 요구사항에 유연하게 대응하기 위해 RwLock으로 감싸서 저장
    lmstudio: Arc<RwLock<OpenAICompatAdapter>>,
    custom_adapters: std::collections::HashMap<String, Arc<dyn ProviderAdapter>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            openai: Arc::new(OpenAICompatAdapter::new(
                "https://api.openai.com/v1".to_string(),
            )),
            xai: Arc::new(OpenAICompatAdapter::new("https://api.x.ai/v1".to_string())),
            openrouter: Arc::new(OpenAICompatAdapter::new(
                "https://openrouter.ai/api/v1".to_string(),
            )),
            anthropic: Arc::new(crate::providers::anthropic::AnthropicAdapter::new(
                "https://api.anthropic.com/v1".to_string(),
            )),
            google: Arc::new(GeminiAdapter::new()),
            // [v3.7.2] LM Studio 로컬 기본 엔드포인트("http://localhost:1234/v1")와 인증 생략(None) 전략 바인딩
            lmstudio: Arc::new(RwLock::new(OpenAICompatAdapter::with_auth(
                "http://localhost:1234/v1".to_string(),
                AuthStrategy::None,
            ))),
            custom_adapters: std::collections::HashMap::new(),
        }
    }

    pub fn register_custom_providers(
        &mut self,
        configs: &[crate::domain::provider::CustomProviderConfig],
    ) -> Result<(), ProviderError> {
        let mut next = std::collections::HashMap::new();
        for config in configs {
            if config.id.is_empty()
                || config.id.len() > 64
                || !config
                    .id
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
            {
                return Err(ProviderError::Configuration(format!(
                    "custom provider id가 유효하지 않습니다: {}",
                    config.id
                )));
            }
            if next.contains_key(&config.id) {
                return Err(ProviderError::Configuration(format!(
                    "custom provider id가 중복되었습니다: {}",
                    config.id
                )));
            }
            let parsed_url = reqwest::Url::parse(&config.base_url).map_err(|error| {
                ProviderError::Configuration(format!(
                    "custom provider base_url이 유효하지 않습니다 ({}): {error}",
                    config.id
                ))
            })?;
            if !matches!(parsed_url.scheme(), "http" | "https")
                || !parsed_url.username().is_empty()
                || parsed_url.password().is_some()
                || parsed_url.query().is_some()
                || parsed_url.fragment().is_some()
                || parsed_url.host_str().is_none()
            {
                return Err(ProviderError::Configuration(format!(
                    "custom provider base_url은 credential/query/fragment 없는 http(s) origin이어야 합니다: {}",
                    config.id
                )));
            }

            let auth_strategy = match config.auth_type.to_lowercase().as_str() {
                "none" => AuthStrategy::None,
                "customheader" => {
                    let header_name = config.auth_header_name.clone().ok_or_else(|| {
                        ProviderError::Configuration(format!(
                            "CustomHeader에는 auth_header_name이 필요합니다: {}",
                            config.id
                        ))
                    })?;
                    reqwest::header::HeaderName::from_bytes(header_name.as_bytes()).map_err(
                        |_| {
                            ProviderError::Configuration(format!(
                                "auth_header_name이 유효하지 않습니다: {}",
                                config.id
                            ))
                        },
                    )?;
                    AuthStrategy::CustomHeader(header_name)
                }
                "bearer" => AuthStrategy::Bearer,
                _ => {
                    return Err(ProviderError::Configuration(format!(
                        "지원하지 않는 custom auth_type입니다: {}",
                        config.auth_type
                    )));
                }
            };

            let adapter: Arc<dyn ProviderAdapter> = match config.dialect {
                crate::domain::provider::ToolDialect::OpenAICompat => {
                    Arc::new(OpenAICompatAdapter::with_auth(
                        config.base_url.trim_end_matches('/').to_string(),
                        auth_strategy,
                    ))
                }
                crate::domain::provider::ToolDialect::Anthropic => {
                    Arc::new(crate::providers::anthropic::AnthropicAdapter::new(
                        config.base_url.trim_end_matches('/').to_string(),
                    ))
                }
                crate::domain::provider::ToolDialect::Gemini => {
                    return Err(ProviderError::Configuration(format!(
                        "custom Gemini dialect는 configured base_url을 보장할 수 없어 지원하지 않습니다: {}",
                        config.id
                    )));
                }
            };
            next.insert(config.id.clone(), adapter);
        }
        self.custom_adapters = next;
        Ok(())
    }

    // [v3.7.2] LM Studio의 base_url을 실시간 갱신하는 런타임 제어 함수 구현
    pub fn update_lmstudio_base_url(&self, url: &str) {
        if let Ok(mut lock) = self.lmstudio.write() {
            *lock = OpenAICompatAdapter::with_auth(url.to_string(), AuthStrategy::None);
        }
    }

    // [v2.5.0] cfg별 분리 구현으로 #[allow(unused_variables)] 제거
    #[cfg(test)]
    pub fn get_adapter(&self, kind: &ProviderKind) -> Arc<dyn ProviderAdapter> {
        if let ProviderKind::Custom(id) = kind
            && !self.custom_adapters.contains_key(id)
        {
            return Arc::new(FailClosedProviderAdapter::new(format!(
                "custom provider adapter가 등록되지 않았습니다: {id}"
            )));
        }
        Arc::new(MockProvider)
    }

    #[cfg(not(test))]
    pub fn get_adapter(&self, kind: &ProviderKind) -> Arc<dyn ProviderAdapter> {
        match kind {
            ProviderKind::OpenAI => self.openai.clone(),
            ProviderKind::Xai => self.xai.clone(),
            ProviderKind::OpenRouter => self.openrouter.clone(),
            ProviderKind::Anthropic => self.anthropic.clone(),
            ProviderKind::Google => self.google.clone(),
            // [v3.7.2] LmStudio 요청 시 RwLock 보관 중인 실시간 어댑터 클론 전달
            ProviderKind::LmStudio => {
                let adapter = self.lmstudio.read().unwrap().clone();
                Arc::new(adapter)
            }
            ProviderKind::Custom(id) => {
                self.custom_adapters.get(id).cloned().unwrap_or_else(|| {
                    Arc::new(FailClosedProviderAdapter::new(format!(
                        "custom provider adapter가 등록되지 않았습니다: {id}"
                    )))
                })
            }
        }
    }
}

// [v1.6.0] 단위 테스트를 위한 MockProvider 구현 (DI 적용)
#[cfg(test)]
pub struct MockProvider;

#[cfg(test)]
impl ProviderAdapter for MockProvider {
    fn validate_credentials<'a>(
        &'a self,
        _api_key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), ProviderError>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }

    fn chat<'a>(
        &'a self,
        _api_key: &'a str,
        _req: crate::providers::types::ChatRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<crate::providers::types::ChatResponse, ProviderError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            Ok(crate::providers::types::ChatResponse {
                message: crate::providers::types::ChatMessage {
                    role: crate::providers::types::Role::Assistant,
                    content: Some("Mock response".to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                    pinned: false,
                },
                input_tokens: 0,
                output_tokens: 0,
            })
        })
    }

    fn chat_stream<'a>(
        &'a self,
        _api_key: &'a str,
        _req: crate::providers::types::ChatRequest,
        delta_tx: tokio::sync::mpsc::Sender<String>,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<crate::providers::types::ChatResponse, ProviderError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            let _ = delta_tx.send("Mock response".to_string()).await;
            Ok(crate::providers::types::ChatResponse {
                message: crate::providers::types::ChatMessage {
                    role: crate::providers::types::Role::Assistant,
                    content: Some("Mock response".to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                    pinned: false,
                },
                input_tokens: 0,
                output_tokens: 0,
            })
        })
    }

    fn fetch_models<'a>(
        &'a self,
        _api_key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, ProviderError>> + Send + 'a>> {
        Box::pin(async move { Ok(vec!["mock-model-1".to_string(), "mock-model-2".to_string()]) })
    }
}

static GLOBAL_PROVIDER_REGISTRY: OnceLock<RwLock<ProviderRegistry>> = OnceLock::new();

fn get_registry() -> &'static RwLock<ProviderRegistry> {
    GLOBAL_PROVIDER_REGISTRY.get_or_init(|| RwLock::new(ProviderRegistry::new()))
}

pub fn get_adapter(kind: &ProviderKind) -> Arc<dyn ProviderAdapter> {
    get_registry().read().unwrap().get_adapter(kind)
}

pub fn reload_providers(
    settings: &crate::domain::settings::PersistedSettings,
) -> Result<(), ProviderError> {
    let mut next = ProviderRegistry::new();
    next.register_custom_providers(&settings.custom_providers)?;
    if let Some(base_url) = &settings.lmstudio_base_url {
        next.update_lmstudio_base_url(base_url);
    }
    *get_registry().write().unwrap() = next;
    Ok(())
}

pub fn update_custom_providers(
    configs: &[crate::domain::provider::CustomProviderConfig],
) -> Result<(), ProviderError> {
    get_registry()
        .write()
        .unwrap()
        .register_custom_providers(configs)
}

// [v3.7.2] 전역 레지스트리에 LM Studio base_url을 바인딩하기 위한 공용 인터페이스
pub fn update_lmstudio_base_url(url: &str) {
    get_registry().read().unwrap().update_lmstudio_base_url(url);
}
