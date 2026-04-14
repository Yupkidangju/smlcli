use anyhow::{Context, Result};
use std::fs;
use crate::domain::tool_result::ToolResult;
use similar::{ChangeTag, TextDiff};

pub(crate) fn read_file(path: &str, start_line: Option<usize>, end_line: Option<usize>) -> Result<ToolResult> {
    let content = fs::read_to_string(path).context("Failed to read file")?;
    let lines: Vec<&str> = content.lines().collect();

    let start = start_line.unwrap_or(1).saturating_sub(1);
    let end = end_line.unwrap_or(lines.len()).min(lines.len());
    let slice = &lines[start..end];
    
    let output = slice.join("\n");
    
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
        Ok(_) => {
            match fs::rename(&tmp_path, path) {
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
            }
        },
        Err(e) => Ok(ToolResult {
            tool_name: "WriteFile".to_string(),
            stdout: String::new(),
            stderr: format!("Failed to write temp file: {}", e),
            exit_code: 1,
            is_error: true,
        })
    }
}
