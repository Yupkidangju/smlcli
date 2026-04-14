use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "tool")]
pub enum ToolCall {
    ReadFile { path: String, start_line: Option<usize>, end_line: Option<usize> },
    ListDir { path: String, depth: Option<usize> },
    WriteFile { path: String, content: String, overwrite: bool },
    ReplaceFileContent { path: String, target_content: String, replacement_content: String },
    ExecShell { command: String, cwd: Option<String>, safe_to_auto_run: bool },
    GrepSearch { pattern: String, path: String, case_insensitive: bool },
    Stat { path: String },
    SysInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolResult {
    pub tool_name: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub is_error: bool,
}
