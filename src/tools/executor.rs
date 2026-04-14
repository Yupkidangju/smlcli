use crate::domain::tool_result::{ToolCall, ToolResult};
use anyhow::Result;
use crate::domain::permissions::PermissionToken;

pub async fn execute_tool(call: ToolCall, _token: &PermissionToken) -> Result<ToolResult> {
    match call {
        ToolCall::ExecShell { command, cwd, safe_to_auto_run: _ } => {
            super::shell::execute_shell(&command, cwd.as_deref()).await
        }
        ToolCall::ReadFile { path, start_line, end_line } => {
            super::file_ops::read_file(&path, start_line, end_line)
        }
        ToolCall::WriteFile { path, content, overwrite: _ } => {
            super::file_ops::write_file_commit(&path, &content)
        }
        ToolCall::ListDir { path, depth } => {
            super::sys_ops::list_dir(&path, depth)
        }
        ToolCall::SysInfo => {
            super::sys_ops::sys_info()
        }
        ToolCall::GrepSearch { pattern, path, case_insensitive } => {
            super::grep::grep_search(&pattern, &path, case_insensitive)
        }
        _ => Ok(ToolResult {
            tool_name: "Unknown".to_string(),
            stdout: String::new(),
            stderr: "Unsupported tool call".to_string(),
            exit_code: 1,
            is_error: true,
        })
    }
}
