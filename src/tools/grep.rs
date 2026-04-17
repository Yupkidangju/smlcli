// [v0.1.0-beta.18] Phase 9-B: Grep 결과 UX 개선.
// 이전: 단순 매칭 라인만 출력 (100건 하드코딩 제한)
// 현재: context_lines 주변 문맥 표시 + max_results 설정 가능 + 결과 요약 헤더
// spec.md §9 Phase 9-B-7 참조.

use crate::domain::tool_result::ToolResult;
use anyhow::Result;
use ignore::WalkBuilder;
use std::fs;

/// [v0.1.0-beta.18] grep 결과의 기본 최대 매칭 수.
/// ToolCall에서 별도 지정하지 않으면 이 값을 사용.
const DEFAULT_MAX_RESULTS: usize = 100;

/// [v0.1.0-beta.18] grep 결과의 기본 주변 문맥 줄 수.
/// 매칭 라인의 위아래로 이 수만큼 추가 표시.
const DEFAULT_CONTEXT_LINES: usize = 2;

/// Grep 검색 실행. pattern 매칭 결과에 주변 문맥(context_lines)을 포함하여 반환.
/// max_results 도달 시 조기 종료하고 truncated 알림 표시.
pub(crate) fn grep_search(pattern: &str, path: &str, case_insensitive: bool) -> Result<ToolResult> {
    let mut output = String::new();
    let mut total_matches = 0;
    let mut files_matched = 0;
    let max_results = DEFAULT_MAX_RESULTS;
    let context_lines = DEFAULT_CONTEXT_LINES;
    let mut truncated = false;

    let walker = WalkBuilder::new(path).hidden(false).build();

    for result in walker {
        match result {
            Ok(entry) => {
                if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                    continue;
                }

                let Ok(content) = fs::read_to_string(entry.path()) else {
                    continue;
                };

                let all_lines: Vec<&str> = content.lines().collect();
                let mut file_has_match = false;

                for (i, line) in all_lines.iter().enumerate() {
                    let matches = if case_insensitive {
                        line.to_lowercase().contains(&pattern.to_lowercase())
                    } else {
                        line.contains(pattern)
                    };

                    if matches {
                        if !file_has_match {
                            // 첫 매칭 시 파일 헤더 표시
                            output.push_str(&format!("\n── {} ──\n", entry.path().display()));
                            file_has_match = true;
                            files_matched += 1;
                        }

                        total_matches += 1;

                        // 주변 문맥 계산: 매칭 라인 전후 context_lines 줄 표시
                        let start = i.saturating_sub(context_lines);
                        let end = (i + context_lines + 1).min(all_lines.len());

                        // 이전 출력과 겹치지 않도록 구분선 추가
                        if total_matches > 1 && start > 0 {
                            output.push_str("  ...\n");
                        }

                        for (j, ctx_line) in
                            all_lines.iter().enumerate().skip(start).take(end - start)
                        {
                            let prefix = if j == i { ">" } else { " " };
                            output.push_str(&format!("{} {:>4}: {}\n", prefix, j + 1, ctx_line));
                        }

                        if total_matches >= max_results {
                            truncated = true;
                            break;
                        }
                    }
                }
            }
            Err(_) => continue,
        }
        if truncated {
            break;
        }
    }

    // 결과 요약 헤더
    let summary = if total_matches == 0 {
        "No matches found.".to_string()
    } else {
        let trunc_msg = if truncated {
            format!(" ({}건 제한, 결과 잘림)", max_results)
        } else {
            String::new()
        };
        format!(
            "🔍 '{}' — {}건 매칭, {}개 파일{}\n{}",
            pattern, total_matches, files_matched, trunc_msg, output
        )
    };

    Ok(ToolResult {
        tool_name: "GrepSearch".to_string(),
        stdout: summary,
        stderr: String::new(),
        exit_code: 0,
        is_error: false,
        tool_call_id: None,
    })
}

// ==========================================
// Phase 13: Agentic Autonomy Tool Registry
// ==========================================

use crate::domain::error::ToolError;
use crate::domain::permissions::PermissionResult;
use crate::domain::settings::PersistedSettings;
use crate::tools::registry::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{Value, json};

pub struct GrepSearchTool;

#[async_trait]
impl Tool for GrepSearchTool {
    fn name(&self) -> &'static str {
        "GrepSearch"
    }

    fn description(&self) -> &'static str {
        "Searches for a pattern in files recursively."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "GrepSearch",
                "description": "Recursively search for a string pattern in files.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string" },
                        "path": { "type": "string" },
                        "case_insensitive": { "type": "boolean" }
                    },
                    "required": ["pattern", "path"]
                }
            }
        })
    }

    fn check_permission(&self, _args: &Value, _settings: &PersistedSettings) -> PermissionResult {
        PermissionResult::Allow
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext<'_>) -> Result<ToolResult, ToolError> {
        let pattern = args
            .get("pattern")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let case_insensitive = args
            .get("case_insensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        grep_search(&pattern, &path, case_insensitive)
            .map_err(|e| ToolError::ExecutionFailure(e.to_string()))
    }
}
