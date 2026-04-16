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
            }),
            Err(e) => {
                let _ = fs::remove_file(&tmp_path);
                Ok(ToolResult {
                    tool_name: "WriteFile".to_string(),
                    stdout: String::new(),
                    stderr: format!("Failed atomic rename: {}", e),
                    exit_code: 1,
                    is_error: true,
                })
            }
        },
        Err(e) => Ok(ToolResult {
            tool_name: "WriteFile".to_string(),
            stdout: String::new(),
            stderr: format!("Failed to write temp file: {}", e),
            exit_code: 1,
            is_error: true,
        }),
    }
}
