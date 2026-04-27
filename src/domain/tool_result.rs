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
    #[serde(default)]
    pub is_truncated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub original_size_bytes: Option<usize>,
    /// [v2.5.1] 도구 실행으로 영향받은 파일 경로 목록.
    /// Git auto-commit 시 이 파일들만 선택적으로 stage하여 사용자 WIP를 보호.
    /// 빈 벡터이면 auto-commit을 스킵.
    #[serde(default)]
    pub affected_paths: Vec<String>,
}
