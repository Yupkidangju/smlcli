use crate::domain::error::ProviderError;
use crate::providers::registry::ProviderAdapter;
use reqwest::Client;
use std::future::Future;
use std::pin::Pin;

pub struct AnthropicAdapter {
    client: Client,
    base_url: String,
}

impl AnthropicAdapter {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }
}

impl ProviderAdapter for AnthropicAdapter {
    fn validate_credentials<'a>(
        &'a self,
        api_key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), ProviderError>> + Send + 'a>> {
        Box::pin(async move {
            // Anthropic doesn't have a simple auth check endpoint, so we fetch models or make a small request.
            let response = self
                .client
                .get(format!("{}/models", self.base_url))
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .send()
                .await
                .map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;

            if response.status().is_success() {
                // Requires exact 2xx success as per Anthropic API documentation.
                Ok(())
            } else {
                Err(ProviderError::AuthenticationFailed(
                    "Invalid Anthropic API Key".into(),
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
            use crate::providers::types::{ChatMessage, Role};

            let mut system_text = String::new();
            let mut anthropic_messages = Vec::new();

            for msg in &req.messages {
                match msg.role {
                    Role::System => {
                        if let Some(content) = &msg.content {
                            if !system_text.is_empty() {
                                system_text.push_str("\n\n");
                            }
                            system_text.push_str(content);
                        }
                    }
                    Role::User => {
                        anthropic_messages.push(serde_json::json!({
                            "role": "user",
                            "content": [{
                                "type": "text",
                                "text": msg.content.clone().unwrap_or_default()
                            }]
                        }));
                    }
                    Role::Assistant => {
                        let mut content_arr = Vec::new();
                        if let Some(text) = &msg.content
                            && !text.is_empty()
                        {
                            content_arr.push(serde_json::json!({
                                "type": "text",
                                "text": text
                            }));
                        }
                        if let Some(tool_calls) = &msg.tool_calls {
                            for tc in tool_calls {
                                let input_json: serde_json::Value =
                                    serde_json::from_str(&tc.function.arguments)
                                        .unwrap_or(serde_json::json!({}));
                                content_arr.push(serde_json::json!({
                                    "type": "tool_use",
                                    "id": tc.id,
                                    "name": tc.function.name,
                                    "input": input_json
                                }));
                            }
                        }
                        anthropic_messages.push(serde_json::json!({
                            "role": "assistant",
                            "content": content_arr
                        }));
                    }
                    Role::Tool => {
                        let content_str = msg.content.clone().unwrap_or_default();
                        let tool_use_id = msg.tool_call_id.clone().unwrap_or_default();
                        anthropic_messages.push(serde_json::json!({
                            "role": "user",
                            "content": [
                                {
                                    "type": "tool_result",
                                    "tool_use_id": tool_use_id,
                                    "content": content_str
                                }
                            ]
                        }));
                    }
                }
            }

            // [v1.8.0] Phase 26: 연속된 같은 역할(role)의 메시지 병합 (찌꺼기/호환성 정제)
            let mut merged_messages: Vec<serde_json::Value> = Vec::new();
            for anthropic_msg in anthropic_messages {
                if let Some(last) = merged_messages.last_mut()
                    && last["role"] == anthropic_msg["role"]
                {
                    // 같은 role이면 content 배열을 병합
                    let mut last_content = last["content"].as_array().unwrap_or(&vec![]).clone();
                    let mut new_content = anthropic_msg["content"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .clone();
                    last_content.append(&mut new_content);
                    last["content"] = serde_json::Value::Array(last_content);
                    continue;
                }
                merged_messages.push(anthropic_msg);
            }

            let mut payload = serde_json::json!({
                "model": req.model,
                "messages": merged_messages,
                "max_tokens": 4096,
                "stream": false,
            });

            if !system_text.is_empty() {
                payload["system"] = serde_json::Value::String(system_text);
            }

            if let Some(tools) = req.tools
                && !tools.is_empty()
            {
                payload["tools"] = serde_json::Value::Array(tools);
            }

            let response = self
                .client
                .post(format!("{}/messages", self.base_url))
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .json(&payload)
                .send()
                .await
                .map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;

            if !response.status().is_success() {
                let code = response.status().as_u16();
                let err_text = response.text().await.unwrap_or_default();
                return Err(ProviderError::ApiResponse {
                    code,
                    message: format!("Anthropic Error: {}", err_text),
                });
            }

            let parsed: serde_json::Value = response
                .json()
                .await
                .map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;
            let mut full_content = String::new();
            let mut tool_calls = Vec::new();

            if let Some(content_arr) = parsed["content"].as_array() {
                for block in content_arr {
                    if block["type"] == "text" {
                        if let Some(t) = block["text"].as_str() {
                            full_content.push_str(t);
                        }
                    } else if block["type"] == "tool_use" {
                        let id = block["id"].as_str().unwrap_or_default().to_string();
                        let name = block["name"].as_str().unwrap_or_default().to_string();
                        let input = block["input"].clone();
                        tool_calls.push(crate::providers::types::ToolCallRequest {
                            id,
                            r#type: "function".to_string(),
                            function: crate::providers::types::FunctionCall {
                                name,
                                arguments: serde_json::to_string(&input).unwrap_or_default(),
                            },
                        });
                    }
                }
            }

            let reply = ChatMessage {
                role: Role::Assistant,
                content: if full_content.is_empty() {
                    None
                } else {
                    Some(full_content)
                },
                tool_calls: if tool_calls.is_empty() {
                    None
                } else {
                    Some(tool_calls)
                },
                tool_call_id: None,
                pinned: false,
            };

            Ok(crate::providers::types::ChatResponse {
                message: reply,
                input_tokens: parsed["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
                output_tokens: parsed["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
            })
        })
    }

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
            use crate::providers::types::{ChatMessage, Role};

            let mut system_text = String::new();
            let mut anthropic_messages = Vec::new();

            for msg in &req.messages {
                match msg.role {
                    Role::System => {
                        if let Some(content) = &msg.content {
                            if !system_text.is_empty() {
                                system_text.push_str("\n\n");
                            }
                            system_text.push_str(content);
                        }
                    }
                    Role::User => {
                        anthropic_messages.push(serde_json::json!({
                            "role": "user",
                            "content": [{
                                "type": "text",
                                "text": msg.content.clone().unwrap_or_default()
                            }]
                        }));
                    }
                    Role::Assistant => {
                        let mut content_arr = Vec::new();
                        if let Some(text) = &msg.content
                            && !text.is_empty()
                        {
                            content_arr.push(serde_json::json!({
                                "type": "text",
                                "text": text
                            }));
                        }
                        if let Some(tool_calls) = &msg.tool_calls {
                            for tc in tool_calls {
                                let input_json: serde_json::Value =
                                    serde_json::from_str(&tc.function.arguments)
                                        .unwrap_or(serde_json::json!({}));
                                content_arr.push(serde_json::json!({
                                    "type": "tool_use",
                                    "id": tc.id,
                                    "name": tc.function.name,
                                    "input": input_json
                                }));
                            }
                        }
                        anthropic_messages.push(serde_json::json!({
                            "role": "assistant",
                            "content": content_arr
                        }));
                    }
                    Role::Tool => {
                        // Anthropic expects tool results as "user" role with type "tool_result"
                        let content_str = msg.content.clone().unwrap_or_default();
                        let tool_use_id = msg.tool_call_id.clone().unwrap_or_default();
                        anthropic_messages.push(serde_json::json!({
                            "role": "user",
                            "content": [
                                {
                                    "type": "tool_result",
                                    "tool_use_id": tool_use_id,
                                    "content": content_str
                                }
                            ]
                        }));
                    }
                }
            }

            // [v1.8.0] Phase 26: 연속된 같은 역할(role)의 메시지 병합 (찌꺼기/호환성 정제)
            let mut merged_messages: Vec<serde_json::Value> = Vec::new();
            for anthropic_msg in anthropic_messages {
                if let Some(last) = merged_messages.last_mut()
                    && last["role"] == anthropic_msg["role"]
                {
                    // 같은 role이면 content 배열을 병합
                    let mut last_content = last["content"].as_array().unwrap_or(&vec![]).clone();
                    let mut new_content = anthropic_msg["content"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .clone();
                    last_content.append(&mut new_content);
                    last["content"] = serde_json::Value::Array(last_content);
                    continue;
                }
                merged_messages.push(anthropic_msg);
            }

            let mut payload = serde_json::json!({
                "model": req.model,
                "messages": merged_messages,
                "max_tokens": 4096,
                "stream": true,
            });

            if !system_text.is_empty() {
                payload["system"] = serde_json::Value::String(system_text);
            }

            if let Some(tools) = req.tools
                && !tools.is_empty()
            {
                payload["tools"] = serde_json::Value::Array(tools);
            }

            let response = self
                .client
                .post(format!("{}/messages", self.base_url))
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .json(&payload)
                .send()
                .await
                .map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;

            if !response.status().is_success() {
                let code = response.status().as_u16();
                let err_text = response.text().await.unwrap_or_default();
                return Err(ProviderError::ApiResponse {
                    code,
                    message: format!("Anthropic Error: {}", err_text),
                });
            }

            let mut full_content = String::new();
            let mut tool_calls_map: std::collections::HashMap<
                usize,
                crate::providers::types::ToolCallRequest,
            > = std::collections::HashMap::new();
            let mut current_tool_idx = 0;

            let body = response
                .text()
                .await
                .map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;
            for line in body.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with(':') {
                    continue;
                }
                if let Some(data) = line.strip_prefix("data: ")
                    && let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data)
                {
                    let event_type = parsed["type"].as_str().unwrap_or("");
                    match event_type {
                        "content_block_delta" => {
                            let delta = &parsed["delta"];
                            if delta["type"] == "text_delta" {
                                if let Some(text) = delta["text"].as_str() {
                                    full_content.push_str(text);
                                    let _ = delta_tx.send(text.to_string()).await;
                                }
                            } else if delta["type"] == "input_json_delta"
                                && let Some(partial_json) = delta["partial_json"].as_str()
                            {
                                let idx =
                                    parsed["index"].as_u64().unwrap_or(current_tool_idx as u64)
                                        as usize;
                                if let Some(tc) = tool_calls_map.get_mut(&idx) {
                                    tc.function.arguments.push_str(partial_json);
                                }
                            }
                        }
                        "content_block_start" => {
                            let cb = &parsed["content_block"];
                            if cb["type"] == "tool_use" {
                                let idx =
                                    parsed["index"].as_u64().unwrap_or(current_tool_idx as u64)
                                        as usize;
                                current_tool_idx = idx;
                                tool_calls_map.insert(
                                    idx,
                                    crate::providers::types::ToolCallRequest {
                                        id: cb["id"].as_str().unwrap_or_default().to_string(),
                                        r#type: "function".to_string(),
                                        function: crate::providers::types::FunctionCall {
                                            name: cb["name"]
                                                .as_str()
                                                .unwrap_or_default()
                                                .to_string(),
                                            arguments: String::new(),
                                        },
                                    },
                                );
                            }
                        }
                        "message_stop" => {
                            break;
                        }
                        _ => {}
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

            let reply = ChatMessage {
                role: Role::Assistant,
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
        _api_key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, ProviderError>> + Send + 'a>> {
        Box::pin(async move {
            // Hardcode 2026.04 models for now since Anthropic /models endpoint is not universally available
            Ok(vec![
                "claude-opus-4-6".to_string(),
                "claude-sonnet-4-6".to_string(),
                "claude-haiku-4-5-20251001".to_string(),
            ])
        })
    }
}
