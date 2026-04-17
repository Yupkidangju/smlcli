use crate::domain::tool_result::ToolResult;
use anyhow::Result;
use std::fs;
use sysinfo::System;

/// [v0.1.0-beta.18] Phase 9-C: 재귀 tree 형태 ListDir.
/// depth 파라미터에 따라 하위 디렉토리를 트리 구조로 표시.
/// 최대 항목 수 제한으로 대규모 디렉토리 과부하 방지.
const MAX_ENTRIES: usize = 1000;

pub(crate) fn list_dir(path: &str, depth: Option<usize>) -> Result<ToolResult> {
    let max_depth = depth.unwrap_or(2);
    let mut out = String::new();
    let mut count = 0;

    // 루트 경로 헤더
    out.push_str(&format!("{}/\n", path));
    list_dir_recursive(path, max_depth, 0, "", &mut out, &mut count);

    if count >= MAX_ENTRIES {
        out.push_str(&format!("\n... ({}개 항목 제한으로 생략됨)\n", MAX_ENTRIES));
    }

    Ok(ToolResult {
        tool_name: "ListDir".to_string(),
        stdout: out,
        stderr: String::new(),
        exit_code: 0,
        is_error: false,
        tool_call_id: None,
    })
}

/// 재귀적으로 디렉토리를 탐색하여 트리 형태로 출력.
/// prefix: 현재 깊이에 따른 들여쓰기 접두사 (│, ├──, └── 등).
fn list_dir_recursive(
    path: &str,
    max_depth: usize,
    current_depth: usize,
    prefix: &str,
    out: &mut String,
    count: &mut usize,
) {
    if current_depth >= max_depth || *count >= MAX_ENTRIES {
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    // 정렬: 디렉토리 우선, 이름 순
    let mut items: Vec<_> = entries.flatten().collect();
    items.sort_by(|a, b| {
        let a_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let b_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
        b_dir
            .cmp(&a_dir)
            .then_with(|| a.file_name().cmp(&b.file_name()))
    });

    let total = items.len();
    for (i, entry) in items.into_iter().enumerate() {
        if *count >= MAX_ENTRIES {
            return;
        }
        *count += 1;

        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let is_last = i == total - 1;

        // 트리 connector: 마지막 항목은 └──, 그 외는 ├──
        let connector = if is_last { "└── " } else { "├── " };
        let suffix = if is_dir { "/" } else { "" };

        out.push_str(&format!("{}{}{}{}\n", prefix, connector, name, suffix));

        // 하위 디렉토리 재귀 탐색
        if is_dir {
            let child_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };
            let child_path = format!("{}/{}", path, name);
            list_dir_recursive(
                &child_path,
                max_depth,
                current_depth + 1,
                &child_prefix,
                out,
                count,
            );
        }
    }
}

pub(crate) fn sys_info() -> Result<ToolResult> {
    let mut sys = System::new_all();
    sys.refresh_all();
    let mut out = String::new();
    out.push_str(&format!(
        "OS: {} {}\n",
        System::name().unwrap_or_default(),
        System::os_version().unwrap_or_default()
    ));
    out.push_str(&format!(
        "Memory: {} / {} KB\n",
        sys.used_memory(),
        sys.total_memory()
    ));
    out.push_str(&format!("CPU Count: {}\n", sys.cpus().len()));
    Ok(ToolResult {
        tool_name: "SysInfo".to_string(),
        stdout: out,
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

pub struct ListDirTool;

#[async_trait]
impl Tool for ListDirTool {
    fn name(&self) -> &'static str {
        "ListDir"
    }

    fn description(&self) -> &'static str {
        "Lists directory contents recursively."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "ListDir",
                "description": "List directory contents in a tree format. Returns max 1000 items.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" },
                        "depth": { "type": "integer", "description": "Max depth to recurse. Default 2." }
                    },
                    "required": ["path"]
                }
            }
        })
    }

    fn check_permission(&self, _args: &Value, _settings: &PersistedSettings) -> PermissionResult {
        PermissionResult::Allow
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext<'_>) -> Result<ToolResult, ToolError> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let depth = args
            .get("depth")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        list_dir(&path, depth).map_err(|e| ToolError::ExecutionFailure(e.to_string()))
    }
}

pub struct SysInfoTool;

#[async_trait]
impl Tool for SysInfoTool {
    fn name(&self) -> &'static str {
        "SysInfo"
    }

    fn description(&self) -> &'static str {
        "Returns basic system info."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "SysInfo",
                "description": "Get OS, Memory, and CPU count.",
                "parameters": {
                    "type": "object",
                    "properties": {}
                }
            }
        })
    }

    fn check_permission(&self, _args: &Value, _settings: &PersistedSettings) -> PermissionResult {
        PermissionResult::Allow
    }

    async fn execute(&self, _args: Value, _ctx: &ToolContext<'_>) -> Result<ToolResult, ToolError> {
        sys_info().map_err(|e| ToolError::ExecutionFailure(e.to_string()))
    }
}

pub struct StatTool;

#[async_trait]
impl Tool for StatTool {
    fn name(&self) -> &'static str {
        "Stat"
    }

    fn description(&self) -> &'static str {
        "Gets file or directory metadata."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "Stat",
                "description": "Get file or directory metadata (size, modified time, readonly, type).",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" }
                    },
                    "required": ["path"]
                }
            }
        })
    }

    fn check_permission(&self, _args: &Value, _settings: &PersistedSettings) -> PermissionResult {
        PermissionResult::Allow
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext<'_>) -> Result<ToolResult, ToolError> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        match std::fs::metadata(&path) {
            Ok(meta) => {
                let file_type = if meta.is_dir() {
                    "디렉토리"
                } else if meta.is_symlink() {
                    "심볼릭 링크"
                } else {
                    "파일"
                };
                let size = meta.len();
                let modified = meta
                    .modified()
                    .map(|t| {
                        t.duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    })
                    .unwrap_or(0);
                let readonly = meta.permissions().readonly();

                let info = format!(
                    "경로: {}\n유형: {}\n크기: {} bytes\n수정일: {} (UNIX epoch)\n읽기전용: {}",
                    path, file_type, size, modified, readonly
                );

                Ok(ToolResult {
                    tool_name: "Stat".to_string(),
                    stdout: info,
                    stderr: String::new(),
                    exit_code: 0,
                    is_error: false,
                    tool_call_id: None,
                })
            }
            Err(e) => Ok(ToolResult {
                tool_name: "Stat".to_string(),
                stdout: String::new(),
                stderr: format!("파일 정보 조회 실패: {}", e),
                exit_code: 1,
                is_error: true,
                tool_call_id: None,
            }),
        }
    }
}
