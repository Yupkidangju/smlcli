use crate::domain::tool_result::ToolResult;
use anyhow::{Context, Result};
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::io::Write;

/// [v0.1.0-beta.18] Phase 9-B: 라인 범위 미지정 시 기본 최대 표시 줄 수.
/// 대규모 파일의 전체 출력을 방지하여 토큰 예산을 보호.
const DEFAULT_MAX_LINES: usize = 800;

pub(crate) fn read_file_context(
    path: &str,
    workspace_root: &std::path::Path,
    max_bytes: usize,
) -> std::result::Result<String, String> {
    let canonical_root = std::fs::canonicalize(workspace_root).map_err(|error| {
        format!(
            "workspace root를 canonicalize할 수 없습니다 ({}): {error}",
            workspace_root.display()
        )
    })?;
    let requested = std::path::Path::new(path);
    let candidate = if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        canonical_root.join(requested)
    };
    let canonical = std::fs::canonicalize(&candidate)
        .map_err(|error| format!("파일 경로를 확인할 수 없습니다 ({path}): {error}"))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(format!(
            "workspace 밖 파일 mention은 허용되지 않습니다: {path}"
        ));
    }

    let metadata = std::fs::metadata(&canonical)
        .map_err(|error| format!("파일 metadata 확인 실패 ({path}): {error}"))?;
    if !metadata.is_file() {
        return Err(format!("regular file만 mention할 수 있습니다: {path}"));
    }
    if metadata.len() > max_bytes as u64 {
        return Err(format!(
            "파일 mention 크기 제한을 초과했습니다 ({} > {} bytes): {}",
            metadata.len(),
            max_bytes,
            path
        ));
    }

    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    }
    let file = options
        .open(&canonical)
        .map_err(|error| format!("파일 mention open 실패 ({path}): {error}"))?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    use std::io::Read;
    file.take(max_bytes.saturating_add(1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("파일 mention read 실패 ({path}): {error}"))?;
    if bytes.len() > max_bytes {
        return Err(format!("파일 mention 크기 제한을 초과했습니다: {path}"));
    }
    if bytes.contains(&0) {
        return Err(format!("binary/NUL 파일은 mention할 수 없습니다: {path}"));
    }
    let control_count = bytes
        .iter()
        .filter(|byte| **byte < 32 && !matches!(**byte, b'\n' | b'\r' | b'\t'))
        .count();
    if !bytes.is_empty() && control_count * 10 > bytes.len() * 3 {
        return Err(format!("binary control byte가 많은 파일입니다: {path}"));
    }
    String::from_utf8(bytes).map_err(|_| format!("UTF-8 text file만 mention할 수 있습니다: {path}"))
}

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
    write_file_commit_with_policy(path, new_content, true)
}

fn file_write_error(message: impl Into<String>) -> ToolResult {
    ToolResult {
        tool_name: "WriteFile".to_string(),
        stdout: String::new(),
        stderr: message.into(),
        exit_code: 1,
        is_error: true,
        tool_call_id: None,
        is_truncated: false,
        original_size_bytes: None,
        affected_paths: vec![],
    }
}

fn write_file_commit_with_policy(
    path: &str,
    new_content: &str,
    overwrite: bool,
) -> Result<ToolResult> {
    let canonical = match validate_sandbox(path) {
        Ok(p) => p,
        Err(e) => return Ok(file_write_error(e)),
    };

    let parent = match canonical.parent() {
        Some(parent) if parent.is_dir() => parent,
        _ => return Ok(file_write_error("대상 파일의 parent directory가 없습니다")),
    };
    let existing_metadata = match fs::symlink_metadata(&canonical) {
        Ok(metadata) => {
            if !metadata.file_type().is_file() {
                return Ok(file_write_error(format!(
                    "regular file만 교체할 수 있습니다: {}",
                    canonical.display()
                )));
            }
            Some(metadata)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Ok(file_write_error(format!(
                "대상 metadata 확인 실패: {error}"
            )));
        }
    };
    if existing_metadata.is_some() && !overwrite {
        return Ok(file_write_error(format!(
            "overwrite=false이므로 기존 파일을 덮어쓸 수 없습니다: {}",
            canonical.display()
        )));
    }

    let file_name = canonical
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("file");
    let mut temp_file = None;
    for _ in 0..16 {
        let candidate = parent.join(format!(".{file_name}.smlcli-{}.tmp", uuid::Uuid::new_v4()));
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options
                .mode(0o666)
                .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
        }
        match options.open(&candidate) {
            Ok(file) => {
                temp_file = Some((candidate, file));
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Ok(file_write_error(format!(
                    "exclusive temp file 생성 실패: {error}"
                )));
            }
        }
    }

    let Some((temp_path, mut file)) = temp_file else {
        return Ok(file_write_error(
            "unique temp file 이름을 확보하지 못했습니다",
        ));
    };
    let cleanup = |message: String| {
        let _ = fs::remove_file(&temp_path);
        file_write_error(message)
    };

    if let Err(error) = file.write_all(new_content.as_bytes()) {
        return Ok(cleanup(format!("temp file write 실패: {error}")));
    }
    if let Some(metadata) = &existing_metadata
        && let Err(error) = file.set_permissions(metadata.permissions())
    {
        return Ok(cleanup(format!("기존 file mode 보존 실패: {error}")));
    }
    if let Err(error) = file.sync_all() {
        return Ok(cleanup(format!("temp file sync 실패: {error}")));
    }
    drop(file);

    let publish_result = if overwrite {
        fs::rename(&temp_path, &canonical)
    } else {
        // Same-directory hard link creation is atomic and fails if the final path
        // appeared after validation. This prevents a create-only race clobber.
        fs::hard_link(&temp_path, &canonical).and_then(|_| fs::remove_file(&temp_path))
    };
    if let Err(error) = publish_result {
        let _ = fs::remove_file(&temp_path);
        return Ok(file_write_error(format!("atomic publish 실패: {error}")));
    }
    if let Ok(directory) = fs::File::open(parent) {
        let _ = directory.sync_all();
    }

    let path_str = canonical.to_string_lossy().to_string();
    Ok(ToolResult {
        tool_name: "WriteFile".to_string(),
        stdout: format!("Successfully wrote to {path_str}"),
        stderr: String::new(),
        exit_code: 0,
        is_error: false,
        tool_call_id: None,
        is_truncated: false,
        original_size_bytes: None,
        affected_paths: vec![path_str],
    })
}

fn replace_exactly_once(
    old_content: &str,
    target: &str,
    replacement: &str,
) -> std::result::Result<String, String> {
    if target.is_empty() {
        return Err("target_content는 비어 있을 수 없습니다".to_string());
    }
    let matches = old_content.match_indices(target).count();
    if matches != 1 {
        return Err(format!(
            "target_content는 정확히 1회 일치해야 합니다 (현재 {matches}회)"
        ));
    }
    Ok(old_content.replacen(target, replacement, 1))
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
                    "required": ["path", "content", "overwrite"]
                }
            }
        })
    }

    fn check_permission(&self, args: &Value, settings: &PersistedSettings) -> PermissionResult {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        if let Err(e) = validate_sandbox(path) {
            return PermissionResult::Deny(e);
        }
        let Some(overwrite) = args.get("overwrite").and_then(|v| v.as_bool()) else {
            return PermissionResult::Deny("overwrite boolean이 필요합니다".to_string());
        };
        if !overwrite && std::path::Path::new(path).exists() {
            return PermissionResult::Deny(format!(
                "overwrite=false이므로 기존 파일을 덮어쓸 수 없습니다: {path}"
            ));
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
        let Some(overwrite) = args.get("overwrite").and_then(|v| v.as_bool()) else {
            return Ok(file_write_error("overwrite boolean이 필요합니다"));
        };

        let old_content = std::fs::read_to_string(&path).unwrap_or_default();
        let diff = generate_diff(&old_content, &content);

        match write_file_commit_with_policy(&path, &content, overwrite) {
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
        let new_text = replace_exactly_once(&old_text, target, replacement).ok()?;
        Some(generate_diff(&old_text, &new_text))
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
                let new_content = match replace_exactly_once(&old_content, &target, &replacement) {
                    Ok(content) => content,
                    Err(message) => {
                        return Ok(ToolResult {
                            tool_name: "ReplaceFileContent".to_string(),
                            stdout: String::new(),
                            stderr: format!("{}: {}", message, path),
                            exit_code: 1,
                            is_error: true,
                            tool_call_id: None,
                            is_truncated: false,
                            original_size_bytes: None,
                            affected_paths: vec![],
                        });
                    }
                };
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
