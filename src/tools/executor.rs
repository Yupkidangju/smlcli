use crate::domain::permissions::PermissionToken;
use crate::domain::tool_result::{ToolCall, ToolResult};
use anyhow::Result;

pub async fn execute_tool(call: ToolCall, _token: &PermissionToken) -> Result<ToolResult> {
    match call {
        ToolCall::ExecShell {
            command,
            cwd,
            safe_to_auto_run: _,
        } => super::shell::execute_shell(&command, cwd.as_deref()).await,
        ToolCall::ReadFile {
            path,
            start_line,
            end_line,
        } => super::file_ops::read_file(&path, start_line, end_line),
        ToolCall::WriteFile {
            path,
            content,
            overwrite: _,
        } => super::file_ops::write_file_commit(&path, &content),
        ToolCall::ListDir { path, depth } => super::sys_ops::list_dir(&path, depth),
        ToolCall::SysInfo => super::sys_ops::sys_info(),
        ToolCall::GrepSearch {
            pattern,
            path,
            case_insensitive,
        } => super::grep::grep_search(&pattern, &path, case_insensitive),
        // [v0.1.0-beta.7] H-3: ReplaceFileContent 도구 구현.
        // 시스템 프롬프트에서 모델에게 안내하고 있으나 실행기에서 미지원이던 문제 해결.
        // read → string replace → atomic write 패턴으로 안전하게 파일 수정.
        ToolCall::ReplaceFileContent {
            path,
            target_content,
            replacement_content,
        } => match std::fs::read_to_string(&path) {
            Ok(old_content) => {
                if !old_content.contains(&target_content) {
                    return Ok(ToolResult {
                        tool_name: "ReplaceFileContent".to_string(),
                        stdout: String::new(),
                        stderr: format!("Target content not found in {}", path),
                        exit_code: 1,
                        is_error: true,
                    });
                }
                let new_content = old_content.replace(&target_content, &replacement_content);
                super::file_ops::write_file_commit(&path, &new_content)
            }
            Err(e) => Ok(ToolResult {
                tool_name: "ReplaceFileContent".to_string(),
                stdout: String::new(),
                stderr: format!("Failed to read file: {}", e),
                exit_code: 1,
                is_error: true,
            }),
        },
        _ => Ok(ToolResult {
            tool_name: "Unknown".to_string(),
            stdout: String::new(),
            stderr: "Unsupported tool call".to_string(),
            exit_code: 1,
            is_error: true,
        }),
    }
}
