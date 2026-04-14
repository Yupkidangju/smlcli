use anyhow::Result;
use crate::domain::tool_result::ToolResult;
use tokio::process::Command;

pub(crate) async fn execute_shell(cmd: &str, cwd: Option<&str>) -> Result<ToolResult> {
    let cwd_path = cwd.unwrap_or(".");
    
    let mut command = if cfg!(target_os = "windows") {
        let mut c = Command::new("powershell");
        c.arg("-Command").arg(cmd);
        c
    } else {
        let mut c = Command::new("sh");
        c.arg("-c").arg(cmd);
        c
    };

    command.current_dir(cwd_path);

    match tokio::time::timeout(std::time::Duration::from_secs(30), command.output()).await {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(1);
            
            Ok(ToolResult {
                tool_name: "ExecShell".to_string(),
                stdout,
                stderr,
                exit_code,
                is_error: !output.status.success(),
            })
        }
        Ok(Err(e)) => {
            Ok(ToolResult {
                tool_name: "ExecShell".to_string(),
                stdout: String::new(),
                stderr: format!("Failed to execute process: {}", e),
                exit_code: 1,
                is_error: true,
            })
        }
        Err(_) => {
            Ok(ToolResult {
                tool_name: "ExecShell".to_string(),
                stdout: String::new(),
                stderr: "Process timed out after 30 seconds.".to_string(),
                exit_code: 1,
                is_error: true,
            })
        }
    }
}
