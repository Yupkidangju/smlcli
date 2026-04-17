use crate::domain::tool_result::ToolResult;
use anyhow::{Context, Result};
use similar::{ChangeTag, TextDiff};
use std::fs;

/// [v0.1.0-beta.18] Phase 9-B: 라인 범위 미지정 시 기본 최대 표시 줄 수.
/// 대규모 파일의 전체 출력을 방지하여 토큰 예산을 보호.
const DEFAULT_MAX_LINES: usize = 800;

pub(crate) fn read_file(
    path: &str,
    start_line: Option<usize>,
    end_line: Option<usize>,
) -> Result<ToolResult> {
    // [v0.1.0-beta.18] Phase 9-B: 경로 정규화 — '..' traversal 이중 방어
    // (PermissionEngine에서 1차 차단, 여기서 2차 방어)
    let canonical = std::path::Path::new(path);
    if path.contains("..") {
        return Ok(ToolResult {
            tool_name: "ReadFile".to_string(),
            stdout: String::new(),
            stderr: format!("경로에 '..'이 포함되어 차단됨: {}", path),
            exit_code: 1,
            is_error: true,
            tool_call_id: None,
        });
    }

    let content = fs::read_to_string(canonical).context("Failed to read file")?;
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let start = start_line.unwrap_or(1).saturating_sub(1);
    // [v0.1.0-beta.18] 라인 범위 미지정 시 DEFAULT_MAX_LINES로 상한
    let end = end_line
        .map(|e| e.min(total_lines))
        .unwrap_or_else(|| (start + DEFAULT_MAX_LINES).min(total_lines));
    let slice = &lines[start..end];

    let mut output = slice.join("\n");

    // 잘린 경우 안내 메시지 추가
    if end < total_lines && end_line.is_none() {
        output.push_str(&format!(
            "\n\n--- (총 {}줄 중 {}~{}줄만 표시. 나머지는 start_line/end_line으로 조회) ---",
            total_lines,
            start + 1,
            end
        ));
    }

    Ok(ToolResult {
        tool_name: "ReadFile".to_string(),
        stdout: output,
        stderr: String::new(),
        exit_code: 0,
        is_error: false,
        tool_call_id: None,
    })
}

pub(crate) fn generate_diff(old_text: &str, new_text: &str) -> String {
    let diff = TextDiff::from_lines(old_text, new_text);
    let mut diff_str = String::new();

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "- ",
            ChangeTag::Insert => "+ ",
            ChangeTag::Equal => "  ",
        };
        diff_str.push_str(&format!("{}{}", sign, change));
    }
    diff_str
}

pub(crate) fn write_file_preview(path: &str, new_content: &str) -> Result<String> {
    let old_content = fs::read_to_string(path).unwrap_or_default();
    Ok(generate_diff(&old_content, new_content))
}

pub(crate) fn write_file_commit(path: &str, new_content: &str) -> Result<ToolResult> {
    let tmp_path = format!("{}.tmp", path);
    match fs::write(&tmp_path, new_content) {
        Ok(_) => match fs::rename(&tmp_path, path) {
            Ok(_) => Ok(ToolResult {
                tool_name: "WriteFile".to_string(),
                stdout: format!("Successfully wrote to {}", path),
                stderr: String::new(),
                exit_code: 0,
                is_error: false,
                tool_call_id: None,
            }),
            Err(e) => {
                let _ = fs::remove_file(&tmp_path);
                Ok(ToolResult {
                    tool_name: "WriteFile".to_string(),
                    stdout: String::new(),
                    stderr: format!("Failed atomic rename: {}", e),
                    exit_code: 1,
                    is_error: true,
                    tool_call_id: None,
                })
            }
        },
        Err(e) => Ok(ToolResult {
            tool_name: "WriteFile".to_string(),
            stdout: String::new(),
            stderr: format!("Failed to write temp file: {}", e),
            exit_code: 1,
            is_error: true,
            tool_call_id: None,
        }),
    }
}

// ==========================================
// Phase 13: Agentic Autonomy Tool Registry
// ==========================================

use crate::domain::error::ToolError;
use crate::domain::permissions::{FileWritePolicy, PermissionResult};
use crate::domain::settings::PersistedSettings;
use crate::tools::registry::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{Value, json};

pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &'static str {
        "ReadFile"
    }

    fn description(&self) -> &'static str {
        "Reads the contents of a file."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "ReadFile",
                "description": "Read file contents with optional line boundaries. Limits to 800 lines if no boundaries are provided.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Absolute or relative path to the file" },
                        "start_line": { "type": "integer", "description": "Line number to start reading from (1-indexed)" },
                        "end_line": { "type": "integer", "description": "Line number to stop reading at" }
                    },
                    "required": ["path"]
                }
            }
        })
    }

    fn check_permission(&self, args: &Value, _settings: &PersistedSettings) -> PermissionResult {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let canonical = std::path::Path::new(path);
        if path.contains("..") {
            return PermissionResult::Deny(format!(
                "경로에 '..'이 포함되어 있어 차단됩니다: {}",
                path
            ));
        }
        if let Ok(meta) = std::fs::metadata(canonical) {
            if meta.len() > 1_048_576 {
                return PermissionResult::Deny(format!(
                    "파일이 1MB를 초과합니다 ({} bytes): {}",
                    meta.len(),
                    path
                ));
            }
        }
        PermissionResult::Allow
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext<'_>) -> Result<ToolResult, ToolError> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let start_line = args
            .get("start_line")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        let end_line = args
            .get("end_line")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        read_file(&path, start_line, end_line)
            .map_err(|e| ToolError::ExecutionFailure(e.to_string()))
    }
}

pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &'static str {
        "WriteFile"
    }

    fn description(&self) -> &'static str {
        "Creates or overwrites a file with content."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "WriteFile",
                "description": "Create a new file or completely overwrite an existing file.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" },
                        "content": { "type": "string" },
                        "overwrite": { "type": "boolean" }
                    },
                    "required": ["path", "content"]
                }
            }
        })
    }

    fn check_permission(&self, _args: &Value, settings: &PersistedSettings) -> PermissionResult {
        match settings.file_write_policy {
            FileWritePolicy::AlwaysAsk => PermissionResult::Ask,
            FileWritePolicy::SessionAllow => PermissionResult::Allow,
        }
    }

    fn format_detail(&self, args: &Value) -> String {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let overwrite = args
            .get("overwrite")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let action = if overwrite { "덮어쓰기" } else { "생성" };
        format!("승인 대기 (y/n) — 파일 {}: {}", action, path)
    }

    fn generate_diff_preview(&self, args: &Value) -> Option<String> {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
        write_file_preview(path, content).ok()
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext<'_>) -> Result<ToolResult, ToolError> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let content = args
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        write_file_commit(&path, &content).map_err(|e| ToolError::ExecutionFailure(e.to_string()))
    }
}

pub struct ReplaceFileContentTool;

#[async_trait]
impl Tool for ReplaceFileContentTool {
    fn name(&self) -> &'static str {
        "ReplaceFileContent"
    }

    fn description(&self) -> &'static str {
        "Replaces a specific block of text in a file."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "ReplaceFileContent",
                "description": "Replace a specific contiguous block of text in an existing file.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" },
                        "target_content": { "type": "string" },
                        "replacement_content": { "type": "string" }
                    },
                    "required": ["path", "target_content", "replacement_content"]
                }
            }
        })
    }

    fn check_permission(&self, _args: &Value, settings: &PersistedSettings) -> PermissionResult {
        match settings.file_write_policy {
            FileWritePolicy::AlwaysAsk => PermissionResult::Ask,
            FileWritePolicy::SessionAllow => PermissionResult::Allow,
        }
    }

    fn format_detail(&self, args: &Value) -> String {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        format!("승인 대기 (y/n) — 파일 수정: {}", path)
    }

    fn generate_diff_preview(&self, args: &Value) -> Option<String> {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let target = args
            .get("target_content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let replacement = args
            .get("replacement_content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let old_text = std::fs::read_to_string(path).unwrap_or_default();
        Some(generate_diff(
            &old_text,
            &old_text.replace(target, replacement),
        ))
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext<'_>) -> Result<ToolResult, ToolError> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let target = args
            .get("target_content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let replacement = args
            .get("replacement_content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        match std::fs::read_to_string(&path) {
            Ok(old_content) => {
                if !old_content.contains(&target) {
                    return Ok(ToolResult {
                        tool_name: "ReplaceFileContent".to_string(),
                        stdout: String::new(),
                        stderr: format!("Target content not found in {}", path),
                        exit_code: 1,
                        is_error: true,
                        tool_call_id: None,
                    });
                }
                let new_content = old_content.replace(&target, &replacement);
                write_file_commit(&path, &new_content)
                    .map_err(|e| ToolError::ExecutionFailure(e.to_string()))
            }
            Err(e) => Ok(ToolResult {
                tool_name: "ReplaceFileContent".to_string(),
                stdout: String::new(),
                stderr: format!("Failed to read file: {}", e),
                exit_code: 1,
                is_error: true,
                tool_call_id: None,
            }),
        }
    }
}
