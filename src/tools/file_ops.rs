use crate::domain::tool_result::ToolResult;
use anyhow::{Context, Result};
use similar::{ChangeTag, TextDiff};
use std::fs;

/// [v0.1.0-beta.18] Phase 9-B: 라인 범위 미지정 시 기본 최대 표시 줄 수.
/// 대규모 파일의 전체 출력을 방지하여 토큰 예산을 보호.
const DEFAULT_MAX_LINES: usize = 800;

pub(crate) fn validate_sandbox(path: &str) -> std::result::Result<std::path::PathBuf, String> {
    let target = std::path::Path::new(path);
    let mut to_check = target.to_path_buf();

    // canonicalize는 존재하는 경로에만 동작하므로, 존재하지 않으면 부모를 찾음
    let mut resolved = None;
    let mut missing_components = Vec::new();

    while let Some(parent) = to_check.parent() {
        if to_check.exists() {
            if let Ok(canon) = std::fs::canonicalize(&to_check) {
                resolved = Some(canon);
            }
            break;
        } else {
            if let Some(name) = to_check.file_name() {
                missing_components.push(name.to_os_string());
            }
            to_check = parent.to_path_buf();
        }
    }

    // 만약 존재하는 부분이 없다면 현재 디렉토리를 기준으로 삼음
    let base = resolved.unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    });
    let mut final_path = base;
    for comp in missing_components.into_iter().rev() {
        final_path.push(comp);
    }

    // 최종 경로가 워크스페이스 루트 하위인지 확인
    let root = crate::infra::workspace_utils::get_current_workspace_root();
    let canon_root = std::fs::canonicalize(&root).unwrap_or(root);

    if final_path.starts_with(&canon_root) {
        Ok(final_path)
    } else {
        Err(format!(
            "보안상 워크스페이스 외부 파일에 접근할 수 없습니다: {}",
            path
        ))
    }
}

pub(crate) fn read_file(
    path: &str,
    start_line: Option<usize>,
    end_line: Option<usize>,
) -> Result<ToolResult> {
    let canonical = match validate_sandbox(path) {
        Ok(p) => p,
        Err(e) => {
            return Ok(ToolResult {
                tool_name: "ReadFile".to_string(),
                stdout: String::new(),
                stderr: e,
                exit_code: 1,
                is_error: true,
                tool_call_id: None,
                is_truncated: false,
                original_size_bytes: None,
                affected_paths: vec![],
            });
        }
    };

    let mut file = fs::File::open(&canonical).context("Failed to open file")?;

    // 이진 파일 검사 (Phase 26)
    let mut buffer = [0u8; 1024];
    use std::io::Read;
    let n = file.read(&mut buffer).unwrap_or(0);

    if n > 0 {
        let slice = &buffer[..n];
        if slice.contains(&0) {
            return Ok(ToolResult {
                tool_name: "ReadFile".to_string(),
                stdout: String::new(),
                stderr: "이진 파일이므로 표시할 수 없습니다".to_string(),
                exit_code: 1,
                is_error: true,
                tool_call_id: None,
                is_truncated: false,
                original_size_bytes: None,
                affected_paths: vec![],
            });
        }

        let mut non_printable = 0;
        for &b in slice {
            if b < 32 && b != b'\n' && b != b'\r' && b != b'\t' {
                non_printable += 1;
            }
        }

        if non_printable as f64 / n as f64 > 0.3 {
            return Ok(ToolResult {
                tool_name: "ReadFile".to_string(),
                stdout: String::new(),
                stderr: "이진 파일이므로 표시할 수 없습니다".to_string(),
                exit_code: 1,
                is_error: true,
                tool_call_id: None,
                is_truncated: false,
                original_size_bytes: None,
                affected_paths: vec![],
            });
        }
    }

    let bytes = fs::read(&canonical).context("Failed to read file bytes")?;
    let content = String::from_utf8_lossy(&bytes).into_owned();
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let start = start_line.unwrap_or(1).saturating_sub(1).min(total_lines);
    // [v0.1.0-beta.18] 라인 범위 미지정 시 DEFAULT_MAX_LINES로 상한
    let end = end_line
        .map(|e| e.min(total_lines).max(start))
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
        is_truncated: false,
        original_size_bytes: None,
        affected_paths: vec![],
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
    let canonical = match validate_sandbox(path) {
        Ok(p) => p,
        Err(e) => {
            return Ok(ToolResult {
                tool_name: "WriteFile".to_string(),
                stdout: String::new(),
                stderr: e,
                exit_code: 1,
                is_error: true,
                tool_call_id: None,
                is_truncated: false,
                original_size_bytes: None,
                affected_paths: vec![],
            });
        }
    };

    let path_str = canonical.to_string_lossy().to_string();
    let tmp_path = format!("{}.tmp", path_str);
    match fs::write(&tmp_path, new_content) {
        Ok(_) => match fs::rename(&tmp_path, &canonical) {
            Ok(_) => Ok(ToolResult {
                tool_name: "WriteFile".to_string(),
                stdout: format!("Successfully wrote to {}", path_str),
                stderr: String::new(),
                exit_code: 0,
                is_error: false,
                tool_call_id: None,
                is_truncated: false,
                original_size_bytes: None,
                affected_paths: vec![path_str.clone()],
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
                    is_truncated: false,
                    original_size_bytes: None,
                    affected_paths: vec![],
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
            is_truncated: false,
            original_size_bytes: None,
            affected_paths: vec![],
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
        // [v2.5.0] 단일 match로 통합: 이중 validate_sandbox + unwrap() 제거
        match validate_sandbox(path) {
            Err(e) => PermissionResult::Deny(e),
            Ok(canonical) => {
                if let Ok(meta) = std::fs::metadata(&canonical)
                    && meta.len() > 1_048_576
                {
                    return PermissionResult::Deny(format!(
                        "파일이 1MB를 초과합니다 ({} bytes): {}",
                        meta.len(),
                        path
                    ));
                }
                PermissionResult::Allow
            }
        }
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

    fn check_permission(&self, args: &Value, settings: &PersistedSettings) -> PermissionResult {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        if let Err(e) = validate_sandbox(path) {
            return PermissionResult::Deny(e);
        }
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
        let canonical = validate_sandbox(path).ok()?;
        let path_str = canonical.to_string_lossy().to_string();
        write_file_preview(&path_str, content).ok()
    }

    fn is_destructive(&self, _args: &Value) -> bool {
        true
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

        let old_content = std::fs::read_to_string(&path).unwrap_or_default();
        let diff = generate_diff(&old_content, &content);

        match write_file_commit(&path, &content) {
            Ok(mut res) => {
                res.stdout = format!("{}\n{}", diff, res.stdout);
                Ok(res)
            }
            Err(e) => Err(ToolError::ExecutionFailure(e.to_string())),
        }
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

    fn check_permission(&self, args: &Value, settings: &PersistedSettings) -> PermissionResult {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        if let Err(e) = validate_sandbox(path) {
            return PermissionResult::Deny(e);
        }
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

        let canonical = validate_sandbox(path).ok()?;
        let old_text = std::fs::read_to_string(canonical).unwrap_or_default();
        Some(generate_diff(
            &old_text,
            &old_text.replace(target, replacement),
        ))
    }

    fn is_destructive(&self, _args: &Value) -> bool {
        true
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

        let canonical = match validate_sandbox(&path) {
            Ok(p) => p,
            Err(e) => {
                return Ok(ToolResult {
                    tool_name: "ReplaceFileContent".to_string(),
                    stdout: String::new(),
                    stderr: e,
                    exit_code: 1,
                    is_error: true,
                    tool_call_id: None,
                    is_truncated: false,
                    original_size_bytes: None,
                    affected_paths: vec![],
                });
            }
        };

        match std::fs::read_to_string(&canonical) {
            Ok(old_content) => {
                if !old_content.contains(&target) {
                    return Ok(ToolResult {
                        tool_name: "ReplaceFileContent".to_string(),
                        stdout: String::new(),
                        stderr: format!("Target content not found in {}", path),
                        exit_code: 1,
                        is_error: true,
                        tool_call_id: None,
                        is_truncated: false,
                        original_size_bytes: None,
                        affected_paths: vec![],
                    });
                }
                let new_content = old_content.replace(&target, &replacement);
                let diff = generate_diff(&old_content, &new_content);
                match write_file_commit(&path, &new_content) {
                    Ok(mut res) => {
                        res.tool_name = "ReplaceFileContent".to_string();
                        res.stdout = format!("{}\n{}", diff, res.stdout);
                        Ok(res)
                    }
                    Err(e) => Err(ToolError::ExecutionFailure(e.to_string())),
                }
            }
            Err(e) => Ok(ToolResult {
                tool_name: "ReplaceFileContent".to_string(),
                stdout: String::new(),
                stderr: format!("Failed to read file: {}", e),
                exit_code: 1,
                is_error: true,
                tool_call_id: None,
                is_truncated: false,
                original_size_bytes: None,
                affected_paths: vec![],
            }),
        }
    }
}

/// [v3.4.0] Phase 44 Task D-1: DeleteFileTool 구현.
/// 워크스페이스 내 파일 삭제를 수행하는 파괴적(destructive) 도구.
/// - validate_sandbox()를 통한 워크스페이스 외부 접근 원천 차단.
/// - FileWritePolicy에 따른 사용자 승인 필수 (AlwaysAsk → Ask, SessionAllow → Allow).
/// - is_destructive() = true → Git 자동 체크포인트 트리거.
/// - 삭제 전 대상 파일의 존재 및 일반 파일 여부를 검증.
pub struct DeleteFileTool;

#[async_trait]
impl Tool for DeleteFileTool {
    fn name(&self) -> &'static str {
        "DeleteFile"
    }

    fn description(&self) -> &'static str {
        "Deletes a file from the workspace."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "DeleteFile",
                "description": "Delete a file from the workspace. Only regular files within the workspace can be deleted.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Absolute or relative path to the file to delete" }
                    },
                    "required": ["path"]
                }
            }
        })
    }

    /// [v3.4.0] 권한 검사: sandbox 검증 + FileWritePolicy 적용.
    /// 워크스페이스 외부 경로는 Deny, 내부 경로는 정책에 따라 Ask 또는 Allow.
    fn check_permission(&self, args: &Value, settings: &PersistedSettings) -> PermissionResult {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        if let Err(e) = validate_sandbox(path) {
            return PermissionResult::Deny(e);
        }
        match settings.file_write_policy {
            FileWritePolicy::AlwaysAsk => PermissionResult::Ask,
            FileWritePolicy::SessionAllow => PermissionResult::Allow,
        }
    }

    fn format_detail(&self, args: &Value) -> String {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        format!("승인 대기 (y/n) — 파일 삭제: {}", path)
    }

    fn is_destructive(&self, _args: &Value) -> bool {
        true
    }

    /// [v3.4.0] 파일 삭제 실행.
    /// 1) validate_sandbox()로 워크스페이스 경계 재검증 (이중 방어).
    /// 2) 대상이 존재하지 않으면 에러 반환.
    /// 3) 대상이 디렉토리이면 안전을 위해 삭제를 거부 (파일만 허용).
    /// 4) std::fs::remove_file()로 삭제 수행.
    async fn execute(&self, args: Value, _ctx: &ToolContext<'_>) -> Result<ToolResult, ToolError> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let canonical = match validate_sandbox(&path) {
            Ok(p) => p,
            Err(e) => {
                return Ok(ToolResult {
                    tool_name: "DeleteFile".to_string(),
                    stdout: String::new(),
                    stderr: e,
                    exit_code: 1,
                    is_error: true,
                    tool_call_id: None,
                    is_truncated: false,
                    original_size_bytes: None,
                    affected_paths: vec![],
                });
            }
        };

        // 대상 파일 존재 여부 확인
        if !canonical.exists() {
            return Ok(ToolResult {
                tool_name: "DeleteFile".to_string(),
                stdout: String::new(),
                stderr: format!("파일이 존재하지 않습니다: {}", path),
                exit_code: 1,
                is_error: true,
                tool_call_id: None,
                is_truncated: false,
                original_size_bytes: None,
                affected_paths: vec![],
            });
        }

        // 디렉토리 삭제 방지: 일반 파일만 허용
        if canonical.is_dir() {
            return Ok(ToolResult {
                tool_name: "DeleteFile".to_string(),
                stdout: String::new(),
                stderr: format!(
                    "안전상 디렉토리는 삭제할 수 없습니다. 파일만 삭제 가능합니다: {}",
                    path
                ),
                exit_code: 1,
                is_error: true,
                tool_call_id: None,
                is_truncated: false,
                original_size_bytes: None,
                affected_paths: vec![],
            });
        }

        let path_str = canonical.to_string_lossy().to_string();

        // 파일 삭제 수행
        match fs::remove_file(&canonical) {
            Ok(_) => Ok(ToolResult {
                tool_name: "DeleteFile".to_string(),
                stdout: format!("파일이 성공적으로 삭제되었습니다: {}", path_str),
                stderr: String::new(),
                exit_code: 0,
                is_error: false,
                tool_call_id: None,
                is_truncated: false,
                original_size_bytes: None,
                affected_paths: vec![path_str],
            }),
            Err(e) => Ok(ToolResult {
                tool_name: "DeleteFile".to_string(),
                stdout: String::new(),
                stderr: format!("파일 삭제 실패: {}", e),
                exit_code: 1,
                is_error: true,
                tool_call_id: None,
                is_truncated: false,
                original_size_bytes: None,
                affected_paths: vec![],
            }),
        }
    }
}
