use crate::domain::tool_result::ToolResult;
use anyhow::Result;
use std::fs;
use sysinfo::{Disks, Networks, System};

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
        b_dir.cmp(&a_dir).then_with(|| a.file_name().cmp(&b.file_name()))
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
            list_dir_recursive(&child_path, max_depth, current_depth + 1, &child_prefix, out, count);
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
    })
}
