use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub struct ToolCall {
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolResult {
    pub tool_name: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub is_error: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}
