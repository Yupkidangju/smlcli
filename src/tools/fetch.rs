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

    fn check_permission(&self, _args: &Value, settings: &PersistedSettings) -> PermissionResult {
        if settings.network_policy == crate::domain::permissions::NetworkPolicy::Deny {
            PermissionResult::Deny("FetchURL is blocked by NetworkPolicy::Deny.".to_string())
        } else if settings.network_policy == crate::domain::permissions::NetworkPolicy::ProviderOnly {
            PermissionResult::Ask
        } else {
            PermissionResult::Allow
        }
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext<'_>) -> Result<ToolResult, ToolError> {
        let url = args
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if url.is_empty() {
            return Err(ToolError::ExecutionFailure("url parameter is missing or empty".to_string()));
        }
        
        if !url.starts_with("https://") && !url.starts_with("http://") {
            return Err(ToolError::ExecutionFailure("URL must start with http:// or https://".to_string()));
        }

        let mut response = reqwest::get(&url)
            .await
            .map_err(|e| ToolError::ExecutionFailure(format!("Failed to fetch URL: {}", e)))?;

        let mut body_bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|e| ToolError::ExecutionFailure(format!("Failed to read chunk: {}", e)))? {
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
        })
    }
}
