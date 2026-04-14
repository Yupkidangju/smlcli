use anyhow::{Result, Context};
use crate::domain::tool_result::ToolResult;
use ignore::WalkBuilder;
use std::fs;

pub(crate) fn grep_search(pattern: &str, path: &str, case_insensitive: bool) -> Result<ToolResult> {
    let mut matching_lines = String::new();
    let mut count = 0;
    
    let walker = WalkBuilder::new(path).hidden(false).build();
    
    // 심플한 Grep 구현체 (MVP 수준)
    for result in walker {
        match result {
            Ok(entry) => {
                if entry.file_type().map(|t| t.is_file()).unwrap_or(false)
                    && let Ok(content) = fs::read_to_string(entry.path()) {
                        for (i, line) in content.lines().enumerate() {
                            let matches = if case_insensitive {
                                line.to_lowercase().contains(&pattern.to_lowercase())
                            } else {
                                line.contains(pattern)
                            };
                            
                            if matches {
                                count += 1;
                                matching_lines.push_str(&format!("{}:{}: {}\n", entry.path().display(), i + 1, line.trim()));
                            }
                            if count >= 100 {
                                matching_lines.push_str("... (Trunkated after 100 matches)\n");
                                break;
                            }
                        }
                }
            }
            Err(_) => continue,
        }
        if count >= 100 { break; }
    }
    
    if count == 0 {
        matching_lines.push_str("No matches found.");
    }

    Ok(ToolResult {
        tool_name: "GrepSearch".to_string(),
        stdout: matching_lines,
        stderr: String::new(),
        exit_code: 0,
        is_error: false,
    })
}
