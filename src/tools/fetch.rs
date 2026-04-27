use crate::domain::error::ToolError;
use crate::domain::permissions::PermissionResult;
use crate::domain::settings::PersistedSettings;
use crate::domain::tool_result::ToolResult;
use crate::tools::registry::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{Value, json};

pub struct FetchUrlTool;

#[async_trait]
impl Tool for FetchUrlTool {
    fn name(&self) -> &'static str {
        "FetchURL"
    }

    fn description(&self) -> &'static str {
        "Fetch content from a URL via HTTP request."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "FetchURL",
                "description": "Fetch content from a URL via HTTP request. Converts HTML to markdown for readability. Use when extracting text from public pages or documentation.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to fetch content from (must be absolute, e.g. https://...)"
                        }
                    },
                    "required": ["url"]
                }
            }
        })
    }

    // [v2.5.0] ProviderOnly 정책 보안 의미론 정합:
    // ProviderOnly = "오직 LLM 프로바이더 API 엔드포인트만 허용".
    // FetchURL은 사용자가 지정한 임의의 외부 URL을 호출하므로,
    // ProviderOnly 환경에서는 SSRF(Server-Side Request Forgery) 방지를 위해
    // Deny로 처리한다. FetchURL을 사용하려면 AllowAll 정책을 선택해야 한다.
    fn check_permission(&self, _args: &Value, settings: &PersistedSettings) -> PermissionResult {
        match settings.network_policy {
            crate::domain::permissions::NetworkPolicy::AllowAll => PermissionResult::Allow,
            crate::domain::permissions::NetworkPolicy::ProviderOnly => {
                PermissionResult::Deny(
                    "FetchURL은 ProviderOnly 정책에서 차단됩니다. 임의 외부 URL 호출은 SSRF 위험이 있으므로, AllowAll 정책으로 변경 후 사용하세요.".to_string(),
                )
            }
            crate::domain::permissions::NetworkPolicy::Deny => {
                PermissionResult::Deny("FetchURL is blocked by NetworkPolicy::Deny.".to_string())
            }
        }
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext<'_>) -> Result<ToolResult, ToolError> {
        let url = args
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if url.is_empty() {
            return Err(ToolError::ExecutionFailure(
                "url parameter is missing or empty".to_string(),
            ));
        }

        if !url.starts_with("https://") && !url.starts_with("http://") {
            return Err(ToolError::ExecutionFailure(
                "URL must start with http:// or https://".to_string(),
            ));
        }

        let mut response = reqwest::get(&url)
            .await
            .map_err(|e| ToolError::ExecutionFailure(format!("Failed to fetch URL: {}", e)))?;

        let mut body_bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| ToolError::ExecutionFailure(format!("Failed to read chunk: {}", e)))?
        {
            body_bytes.extend_from_slice(&chunk);
            if body_bytes.len() > 500_000 {
                break;
            }
        }

        let content = String::from_utf8_lossy(&body_bytes).to_string();

        let clean_text = html2md::parse_html(&content);

        let mut truncated = clean_text;
        if truncated.len() > 10_000 {
            truncated.truncate(10_000);
            truncated.push_str("\n\n... (Content truncated due to size limit) ...");
        }

        Ok(ToolResult {
            tool_name: "FetchURL".to_string(),
            stdout: truncated,
            stderr: String::new(),
            exit_code: 0,
            is_error: false,
            tool_call_id: None,
            is_truncated: false,
            original_size_bytes: None,
            affected_paths: vec![],
        })
    }
}
