use anyhow::Result;
use crate::domain::tool_result::ToolResult;
use sysinfo::{System, Disks, Networks};
use std::fs;

pub(crate) fn list_dir(path: &str, depth: Option<usize>) -> Result<ToolResult> {
    // 간단한 ListDir (Tree 형태나 깊이 지원은 MVP 이후 상세구현)
    let d = depth.unwrap_or(1);
    let mut out = String::new();
    if d > 0 && let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                out.push_str(&format!("{}{}\n", name, if is_dir { "/" } else { "" }));
            }
    }
    Ok(ToolResult {
        tool_name: "ListDir".to_string(),
        stdout: out,
        stderr: String::new(),
        exit_code: 0,
        is_error: false,
    })
}

pub(crate) fn sys_info() -> Result<ToolResult> {
    let mut sys = System::new_all();
    sys.refresh_all();
    let mut out = String::new();
    out.push_str(&format!("OS: {} {}\n", System::name().unwrap_or_default(), System::os_version().unwrap_or_default()));
    out.push_str(&format!("Memory: {} / {} KB\n", sys.used_memory(), sys.total_memory()));
    out.push_str(&format!("CPU Count: {}\n", sys.cpus().len()));
    Ok(ToolResult {
        tool_name: "SysInfo".to_string(),
        stdout: out,
        stderr: String::new(),
        exit_code: 0,
        is_error: false,
    })
}
