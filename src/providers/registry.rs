use crate::domain::provider::ProviderKind;
use anyhow::Result;
use reqwest::Client;
use std::future::Future;
use std::pin::Pin;

pub trait ProviderAdapter: Send + Sync {
    /// 해당 Provider에 대한 API 인증 정보를 최소한으로 검증하는 smoke test 함수
    fn validate_credentials<'a>(
        &'a self,
        api_key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;

    /// Provider에 맞추어 채팅 요청을 전송하고 응답을 반환
    fn chat<'a>(
        &'a self,
        api_key: &'a str,
        req: crate::providers::types::ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<crate::providers::types::ChatResponse>> + Send + 'a>>;

    /// [v0.1.0-beta.18] Phase 10: SSE 스트리밍 채팅.
    /// 델타 토큰을 tx로 실시간 전송하고, 완료 시 전체 응답을 반환.
    fn chat_stream<'a>(
        &'a self,
        api_key: &'a str,
        req: crate::providers::types::ChatRequest,
        delta_tx: tokio::sync::mpsc::Sender<String>,
    ) -> Pin<Box<dyn Future<Output = Result<crate::providers::types::ChatResponse>> + Send + 'a>>;

    /// 지원하는 모델 목록을 동적으로 가져옴
    fn fetch_models<'a>(
        &'a self,
        api_key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>>> + Send + 'a>>;
}

pub struct OpenRouterAdapter {
    client: Client,
}

impl OpenRouterAdapter {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl ProviderAdapter for OpenRouterAdapter {
    fn validate_credentials<'a>(
        &'a self,
        api_key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let response = self
                .client
                .get("https://openrouter.ai/api/v1/auth/key")
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await?;

            if response.status().is_success() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Invalid OpenRouter API Key"))
            }
        })
    }

    fn chat<'a>(
        &'a self,
        api_key: &'a str,
        req: crate::providers::types::ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<crate::providers::types::ChatResponse>> + Send + 'a>>
    {
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
            let payload = Payload {
                model: &req.model,
                messages: &req.messages,
                tools: &req.tools,
                tool_choice: &req.tool_choice,
            };

            let response = self
                .client
                .post("https://openrouter.ai/api/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&payload)
                .send()
                .await?;

            if !response.status().is_success() {
                let err_text = response.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!("OpenRouter Error: {}", err_text));
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

            let mut parsed: OpenRouterRes = response.json().await?;
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
    ) -> Pin<Box<dyn Future<Output = Result<crate::providers::types::ChatResponse>> + Send + 'a>>
    {
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
                .post("https://openrouter.ai/api/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&payload)
                .send()
                .await?;

            if !response.status().is_success() {
                let err_text = response.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!("OpenRouter Stream Error: {}", err_text));
            }

            // SSE 라인 단위 파싱
            let mut full_content = String::new();
            let mut tool_calls_map: std::collections::HashMap<
                usize,
                crate::providers::types::ToolCallRequest,
            > = std::collections::HashMap::new();
            let body = response.text().await?;
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
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>>> + Send + 'a>> {
        Box::pin(async move {
            let response = self
                .client
                .get("https://openrouter.ai/api/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await?;
            if !response.status().is_success() {
                return Err(anyhow::anyhow!("Failed to fetch OpenRouter models"));
            }
            #[derive(serde::Deserialize)]
            struct ModelObj {
                id: String,
            }
            #[derive(serde::Deserialize)]
            struct ModelRes {
                data: Vec<ModelObj>,
            }

            let parsed: ModelRes = response.json().await?;
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
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                api_key
            );
            let response = self.client.get(&url).send().await?;

            if response.status().is_success() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Invalid Gemini API Key"))
            }
        })
    }

    fn chat<'a>(
        &'a self,
        api_key: &'a str,
        req: crate::providers::types::ChatRequest,
    ) -> Pin<Box<dyn Future<Output = Result<crate::providers::types::ChatResponse>> + Send + 'a>>
    {
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
                .await?;

            if !response.status().is_success() {
                let err_text = response.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!("Gemini Error: {}", err_text));
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
                content: String,
            }

            let mut parsed: GeminiRes = response.json().await?;
            let reply_content = if !parsed.choices.is_empty() {
                parsed.choices.remove(0).message.content
            } else {
                "No response from Gemini API.".to_string()
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

    // [v0.1.0-beta.18] Phase 10: Gemini SSE 스트리밍 (OpenAI 호환 엔드포인트).
    fn chat_stream<'a>(
        &'a self,
        api_key: &'a str,
        req: crate::providers::types::ChatRequest,
        delta_tx: tokio::sync::mpsc::Sender<String>,
    ) -> Pin<Box<dyn Future<Output = Result<crate::providers::types::ChatResponse>> + Send + 'a>>
    {
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
                .await?;

            if !response.status().is_success() {
                let err_text = response.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!("Gemini Stream Error: {}", err_text));
            }

            let mut full_content = String::new();
            let body = response.text().await?;
            for line in body.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with(':') {
                    continue;
                }
                if let Some(data) = line.strip_prefix("data: ") {
                    if data.trim() == "[DONE]" {
                        break;
                    }
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data)
                        && let Some(delta) = parsed["choices"][0]["delta"]["content"].as_str()
                    {
                        full_content.push_str(delta);
                        let _ = delta_tx.send(delta.to_string()).await;
                    }
                }
            }

            let reply = crate::providers::types::ChatMessage {
                role: crate::providers::types::Role::Assistant,
                content: Some(full_content),
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

    fn fetch_models<'a>(
        &'a self,
        api_key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>>> + Send + 'a>> {
        Box::pin(async move {
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                api_key
            );
            let response = self.client.get(&url).send().await?;
            if !response.status().is_success() {
                return Err(anyhow::anyhow!("Failed to fetch Gemini models"));
            }
            #[derive(serde::Deserialize)]
            struct ModelObj {
                name: String,
            }
            #[derive(serde::Deserialize)]
            struct ModelRes {
                models: Vec<ModelObj>,
            }

            let parsed: ModelRes = response.json().await?;
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

pub fn get_adapter(kind: &ProviderKind) -> Box<dyn ProviderAdapter> {
    match kind {
        ProviderKind::OpenRouter => Box::new(OpenRouterAdapter::new()),
        ProviderKind::Google => Box::new(GeminiAdapter::new()),
    }
}
