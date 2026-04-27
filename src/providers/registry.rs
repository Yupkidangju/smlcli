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

pub struct OpenAICompatAdapter {
    client: Client,
    base_url: String,
    auth_strategy: AuthStrategy,
}

impl OpenAICompatAdapter {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            auth_strategy: AuthStrategy::Bearer,
        }
    }

    /// [v2.5.2] 커스텀 인증 전략으로 어댑터 생성
    pub fn with_auth(base_url: String, auth_strategy: AuthStrategy) -> Self {
        Self {
            client: Client::new(),
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
                let err_text = response.text().await.unwrap_or_default();
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

            let mut parsed: OpenRouterRes = response
                .json()
                .await
                .map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;
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
                let err_text = response.text().await.unwrap_or_default();
                return Err(ProviderError::ApiResponse {
                    code,
                    message: format!("Stream API Error: {}", err_text),
                });
            }

            // SSE 라인 단위 파싱
            let mut full_content = String::new();
            let mut tool_calls_map: std::collections::HashMap<
                usize,
                crate::providers::types::ToolCallRequest,
            > = std::collections::HashMap::new();
            let body = response
                .text()
                .await
                .map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;
            for line in body.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with(':') {
                    continue;
                }
                if let Some(data) = line.strip_prefix("data: ") {
                    if data.trim() == "[DONE]" {
                        break;
                    }
                    // SSE delta JSON 파싱
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                        let delta = &parsed["choices"][0]["delta"];
                        if let Some(content) = delta["content"].as_str() {
                            full_content.push_str(content);
                            let _ = delta_tx.send(content.to_string()).await;
                        }
                        if let Some(tc_array) = delta["tool_calls"].as_array() {
                            for tc_val in tc_array {
                                if let Some(idx) = tc_val["index"].as_u64().map(|i| i as usize) {
                                    let entry = tool_calls_map.entry(idx).or_insert_with(|| {
                                        crate::providers::types::ToolCallRequest {
                                            id: tc_val["id"]
                                                .as_str()
                                                .unwrap_or_default()
                                                .to_string(),
                                            r#type: "function".to_string(),
                                            function: crate::providers::types::FunctionCall {
                                                name: tc_val["function"]["name"]
                                                    .as_str()
                                                    .unwrap_or_default()
                                                    .to_string(),
                                                arguments: String::new(),
                                            },
                                        }
                                    });
                                    if let Some(arg_chunk) =
                                        tc_val["function"]["arguments"].as_str()
                                    {
                                        entry.function.arguments.push_str(arg_chunk);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let tool_calls = if tool_calls_map.is_empty() {
                None
            } else {
                let mut tcs: Vec<_> = tool_calls_map.into_iter().collect();
                tcs.sort_by_key(|k| k.0);
                Some(tcs.into_iter().map(|(_, v)| v).collect())
            };

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

            let parsed: ModelRes = response
                .json()
                .await
                .map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;
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
                let err_text = response.text().await.unwrap_or_default();
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

            let mut parsed: GeminiRes = response
                .json()
                .await
                .map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;

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
                let err_text = response.text().await.unwrap_or_default();
                return Err(ProviderError::ApiResponse {
                    code,
                    message: format!("Gemini Stream Error: {}", err_text),
                });
            }

            let mut full_content = String::new();
            let mut tool_calls_map: std::collections::HashMap<
                usize,
                crate::providers::types::ToolCallRequest,
            > = std::collections::HashMap::new();
            let body = response
                .text()
                .await
                .map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;
            for line in body.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with(':') {
                    continue;
                }
                if let Some(data) = line.strip_prefix("data: ") {
                    if data.trim() == "[DONE]" {
                        break;
                    }
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                        let delta = &parsed["choices"][0]["delta"];
                        if let Some(content) = delta["content"].as_str() {
                            full_content.push_str(content);
                            let _ = delta_tx.send(content.to_string()).await;
                        }
                        if let Some(tc_array) = delta["tool_calls"].as_array() {
                            for tc_val in tc_array {
                                if let Some(idx) = tc_val["index"].as_u64().map(|i| i as usize) {
                                    let entry = tool_calls_map.entry(idx).or_insert_with(|| {
                                        crate::providers::types::ToolCallRequest {
                                            id: tc_val["id"]
                                                .as_str()
                                                .unwrap_or_default()
                                                .to_string(),
                                            r#type: "function".to_string(),
                                            function: crate::providers::types::FunctionCall {
                                                name: tc_val["function"]["name"]
                                                    .as_str()
                                                    .unwrap_or_default()
                                                    .to_string(),
                                                arguments: String::new(),
                                            },
                                        }
                                    });
                                    if let Some(arg_chunk) =
                                        tc_val["function"]["arguments"].as_str()
                                    {
                                        entry.function.arguments.push_str(arg_chunk);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let tool_calls = if tool_calls_map.is_empty() {
                None
            } else {
                let mut tcs: Vec<_> = tool_calls_map.into_iter().collect();
                tcs.sort_by_key(|k| k.0);
                Some(tcs.into_iter().map(|(_, v)| v).collect())
            };

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

            let parsed: ModelRes = response
                .json()
                .await
                .map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;
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
            custom_adapters: std::collections::HashMap::new(),
        }
    }

    pub fn register_custom_providers(
        &mut self,
        configs: &[crate::domain::provider::CustomProviderConfig],
    ) {
        for config in configs {
            // [v2.5.2] 감사 HIGH-2: auth_type → AuthStrategy 변환.
            // None이면 헤더 없이, CustomHeader면 지정된 헤더로, 기본은 Bearer.
            let auth_strategy = match config.auth_type.to_lowercase().as_str() {
                "none" => AuthStrategy::None,
                "customheader" => {
                    let header_name = config
                        .auth_header_name
                        .clone()
                        .unwrap_or_else(|| "X-API-Key".to_string());
                    AuthStrategy::CustomHeader(header_name)
                }
                _ => AuthStrategy::Bearer, // "Bearer" 또는 기타
            };

            let adapter: Arc<dyn ProviderAdapter> = match config.dialect {
                crate::domain::provider::ToolDialect::OpenAICompat => Arc::new(
                    OpenAICompatAdapter::with_auth(config.base_url.clone(), auth_strategy),
                ),
                crate::domain::provider::ToolDialect::Anthropic => Arc::new(
                    crate::providers::anthropic::AnthropicAdapter::new(config.base_url.clone()),
                ),
                crate::domain::provider::ToolDialect::Gemini => {
                    Arc::new(GeminiAdapter::new()) // Gemini는 baseUrl 변경이 지원 안 되지만 일단 더미 매핑
                }
            };
            self.custom_adapters.insert(config.id.clone(), adapter);
        }
    }

    // [v2.5.0] cfg별 분리 구현으로 #[allow(unused_variables)] 제거
    #[cfg(test)]
    pub fn get_adapter(&self, _kind: &ProviderKind) -> Arc<dyn ProviderAdapter> {
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
            ProviderKind::Custom(id) => self
                .custom_adapters
                .get(id)
                .cloned()
                .unwrap_or_else(|| self.openai.clone()),
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

pub fn reload_providers() {
    *get_registry().write().unwrap() = ProviderRegistry::new();
}

pub fn update_custom_providers(configs: &[crate::domain::provider::CustomProviderConfig]) {
    get_registry()
        .write()
        .unwrap()
        .register_custom_providers(configs);
}
