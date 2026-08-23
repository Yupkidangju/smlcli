use crate::domain::error::ProviderError;

pub(crate) const MAX_PROVIDER_STREAM_BYTES: usize = 8 * 1024 * 1024;
pub(crate) const MAX_PROVIDER_LINE_BYTES: usize = 1024 * 1024;
pub(crate) const MAX_PROVIDER_CONTENT_BYTES: usize = 5 * 1024 * 1024;
pub(crate) const MAX_TOOL_ARGUMENT_BYTES: usize = 1024 * 1024;
const MAX_ERROR_BODY_BYTES: usize = 64 * 1024;

pub(crate) struct SseDecoder {
    buffer: Vec<u8>,
    total_bytes: usize,
}

impl SseDecoder {
    pub(crate) fn new() -> Self {
        Self {
            buffer: Vec::new(),
            total_bytes: 0,
        }
    }

    pub(crate) fn push(&mut self, chunk: &[u8]) -> Result<Vec<String>, ProviderError> {
        self.total_bytes = self.total_bytes.saturating_add(chunk.len());
        if self.total_bytes > MAX_PROVIDER_STREAM_BYTES {
            return Err(ProviderError::NetworkFailure(format!(
                "provider stream이 {} bytes 제한을 초과했습니다",
                MAX_PROVIDER_STREAM_BYTES
            )));
        }
        self.buffer.extend_from_slice(chunk);
        let mut lines = Vec::new();
        while let Some(index) = self.buffer.iter().position(|byte| *byte == b'\n') {
            if index > MAX_PROVIDER_LINE_BYTES {
                return Err(ProviderError::NetworkFailure(format!(
                    "provider SSE line이 {} bytes 제한을 초과했습니다",
                    MAX_PROVIDER_LINE_BYTES
                )));
            }
            let mut line = self.buffer.drain(..=index).collect::<Vec<_>>();
            if line.last() == Some(&b'\n') {
                line.pop();
            }
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            lines.push(String::from_utf8(line).map_err(|_| {
                ProviderError::NetworkFailure("provider SSE line이 UTF-8이 아닙니다".to_string())
            })?);
        }
        if self.buffer.len() > MAX_PROVIDER_LINE_BYTES {
            return Err(ProviderError::NetworkFailure(format!(
                "provider SSE partial line이 {} bytes 제한을 초과했습니다",
                MAX_PROVIDER_LINE_BYTES
            )));
        }
        Ok(lines)
    }

    pub(crate) fn finish(mut self) -> Result<Vec<String>, ProviderError> {
        if self.buffer.is_empty() {
            return Ok(Vec::new());
        }
        if self.buffer.len() > MAX_PROVIDER_LINE_BYTES {
            return Err(ProviderError::NetworkFailure(
                "provider SSE final line size limit 초과".to_string(),
            ));
        }
        Ok(vec![
            String::from_utf8(std::mem::take(&mut self.buffer)).map_err(|_| {
                ProviderError::NetworkFailure("provider SSE final line UTF-8 오류".to_string())
            })?,
        ])
    }
}

pub(crate) async fn bounded_error_body(
    mut response: reqwest::Response,
    secrets: &[&str],
) -> String {
    let mut bytes = Vec::new();
    while let Ok(Some(chunk)) = response.chunk().await {
        let remaining = MAX_ERROR_BODY_BYTES.saturating_sub(bytes.len());
        if remaining == 0 {
            break;
        }
        let take = remaining.min(chunk.len());
        bytes.extend_from_slice(&chunk[..take]);
        if take < chunk.len() {
            break;
        }
    }
    let body = String::from_utf8_lossy(&bytes);
    crate::infra::redaction::redact_sensitive_text(&body, secrets)
}

pub(crate) async fn bounded_json_response<T>(
    mut response: reqwest::Response,
) -> Result<T, ProviderError>
where
    T: serde::de::DeserializeOwned,
{
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| ProviderError::NetworkFailure(error.to_string()))?
    {
        if bytes.len().saturating_add(chunk.len()) > MAX_PROVIDER_STREAM_BYTES {
            return Err(ProviderError::NetworkFailure(format!(
                "provider JSON response가 {} bytes 제한을 초과했습니다",
                MAX_PROVIDER_STREAM_BYTES
            )));
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|error| {
        ProviderError::NetworkFailure(format!("provider JSON parse 실패: {error}"))
    })
}

pub(crate) async fn openai_compatible_stream(
    mut response: reqwest::Response,
    delta_tx: tokio::sync::mpsc::Sender<String>,
) -> Result<
    (
        String,
        Option<Vec<crate::providers::types::ToolCallRequest>>,
    ),
    ProviderError,
> {
    let mut decoder = SseDecoder::new();
    let mut full_content = String::new();
    let mut tool_calls =
        std::collections::HashMap::<usize, crate::providers::types::ToolCallRequest>::new();

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| ProviderError::NetworkFailure(error.to_string()))?
    {
        for line in decoder.push(&chunk)? {
            if process_openai_line(&line, &mut full_content, &mut tool_calls, &delta_tx).await? {
                return Ok((full_content, sorted_tool_calls(tool_calls)));
            }
        }
    }
    for line in decoder.finish()? {
        let _ = process_openai_line(&line, &mut full_content, &mut tool_calls, &delta_tx).await?;
    }
    Ok((full_content, sorted_tool_calls(tool_calls)))
}

async fn process_openai_line(
    line: &str,
    full_content: &mut String,
    tool_calls: &mut std::collections::HashMap<usize, crate::providers::types::ToolCallRequest>,
    delta_tx: &tokio::sync::mpsc::Sender<String>,
) -> Result<bool, ProviderError> {
    let line = line.trim();
    if line.is_empty() || line.starts_with(':') {
        return Ok(false);
    }
    let Some(data) = line.strip_prefix("data: ") else {
        return Ok(false);
    };
    if data.trim() == "[DONE]" {
        return Ok(true);
    }
    let parsed: serde_json::Value = serde_json::from_str(data)
        .map_err(|error| ProviderError::NetworkFailure(format!("SSE JSON parse 실패: {error}")))?;
    let delta = &parsed["choices"][0]["delta"];
    if let Some(content) = delta["content"].as_str() {
        if full_content.len().saturating_add(content.len()) > MAX_PROVIDER_CONTENT_BYTES {
            return Err(ProviderError::NetworkFailure(
                "provider content size limit 초과".to_string(),
            ));
        }
        full_content.push_str(content);
        let _ = delta_tx.send(content.to_string()).await;
    }
    if let Some(items) = delta["tool_calls"].as_array() {
        for value in items {
            let Some(index) = value["index"].as_u64().map(|index| index as usize) else {
                continue;
            };
            let entry = tool_calls.entry(index).or_insert_with(|| {
                crate::providers::types::ToolCallRequest {
                    id: value["id"].as_str().unwrap_or_default().to_string(),
                    r#type: "function".to_string(),
                    function: crate::providers::types::FunctionCall {
                        name: value["function"]["name"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        arguments: String::new(),
                    },
                }
            });
            if let Some(arguments) = value["function"]["arguments"].as_str() {
                if entry
                    .function
                    .arguments
                    .len()
                    .saturating_add(arguments.len())
                    > MAX_TOOL_ARGUMENT_BYTES
                {
                    return Err(ProviderError::NetworkFailure(
                        "provider tool arguments size limit 초과".to_string(),
                    ));
                }
                entry.function.arguments.push_str(arguments);
            }
        }
    }
    Ok(false)
}

fn sorted_tool_calls(
    tool_calls: std::collections::HashMap<usize, crate::providers::types::ToolCallRequest>,
) -> Option<Vec<crate::providers::types::ToolCallRequest>> {
    if tool_calls.is_empty() {
        return None;
    }
    let mut calls = tool_calls.into_iter().collect::<Vec<_>>();
    calls.sort_by_key(|(index, _)| *index);
    Some(calls.into_iter().map(|(_, call)| call).collect())
}
