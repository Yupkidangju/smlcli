// [v0.1.0-beta.18] Phase 9-C: Shell stdout/stderr 실시간 스트리밍 구현.
// 이전: command.output()으로 전체 완료 후 일괄 수집 (30초 타임아웃)
// 현재: stdout/stderr를 라인 단위로 비동기 스트리밍하여 ToolOutputChunk 이벤트 발행 가능.
//       action_tx가 없는 경우(기존 호출)는 버퍼 모드로 동작하여 하위 호환 유지.
// spec.md §9 Phase 9-C 참조.

use crate::domain::tool_result::ToolResult;
use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// [v0.1.0-beta.18] 셸 실행 타임아웃 (초).
const SHELL_TIMEOUT_SECS: u64 = 30;

/// 셸 명령어를 실행하고 stdout/stderr를 수집하여 ToolResult로 반환.
/// action_tx가 Some이면 각 라인을 ToolOutputChunk 이벤트로 실시간 전송.
pub(crate) async fn execute_shell(cmd: &str, cwd: Option<&str>) -> Result<ToolResult> {
    execute_shell_streaming(cmd, cwd, None).await
}

fn resolve_shell_cwd(cwd: Option<&str>) -> Result<PathBuf> {
    let cwd_path = cwd.unwrap_or(".");
    let host_cwd = Path::new(cwd_path);
    let resolved = if host_cwd.is_absolute() {
        host_cwd.to_path_buf()
    } else {
        std::env::current_dir()?.join(host_cwd)
    };
    resolved
        .canonicalize()
        .map_err(|e| anyhow::anyhow!("작업 디렉터리 확인 실패 ({}): {}", resolved.display(), e))
}

pub fn command_in_path(binary: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join(binary))
        .find(|candidate| candidate.exists())
}

#[cfg(target_os = "linux")]
fn build_linux_sandbox_command(cmd: &str, host_cwd: &Path) -> Result<Command> {
    let Some(bwrap) = command_in_path("bwrap") else {
        return Err(anyhow::anyhow!(
            "Linux 샌드박스 백엔드 'bwrap'을 찾을 수 없습니다. bubblewrap 설치가 필요합니다."
        ));
    };

    let workspace_mount = "/workspace";
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let cargo_home = format!("{}/.cargo", home);
    let rustup_home = format!("{}/.rustup", home);

    let mut command = Command::new(bwrap);
    command
        .arg("--unshare-all")
        .arg("--share-net")
        .arg("--die-with-parent")
        .arg("--new-session")
        .arg("--proc")
        .arg("/proc")
        .arg("--dev")
        .arg("/dev")
        .arg("--ro-bind")
        .arg("/usr")
        .arg("/usr")
        .arg("--ro-bind")
        .arg("/bin")
        .arg("/bin")
        .arg("--ro-bind-try")
        .arg("/lib")
        .arg("/lib")
        .arg("--ro-bind-try")
        .arg("/lib64")
        .arg("/lib64")
        .arg("--ro-bind-try")
        .arg("/sbin")
        .arg("/sbin")
        .arg("--ro-bind")
        .arg("/etc")
        .arg("/etc")
        .arg("--ro-bind-try")
        .arg(&cargo_home)
        .arg(&cargo_home)
        .arg("--ro-bind-try")
        .arg(&rustup_home)
        .arg(&rustup_home)
        .arg("--tmpfs")
        .arg("/tmp")
        .arg("--bind")
        .arg(host_cwd)
        .arg(workspace_mount)
        .arg("--chdir")
        .arg(workspace_mount)
        .arg("--clearenv")
        .arg("--setenv")
        .arg("PATH")
        .arg(format!("{}/bin:/usr/bin:/bin", cargo_home))
        .arg("--setenv")
        .arg("HOME")
        .arg("/tmp")
        .arg("--setenv")
        .arg("CARGO_HOME")
        .arg(&cargo_home)
        .arg("--setenv")
        .arg("RUSTUP_HOME")
        .arg(&rustup_home)
        .arg("sh")
        .arg("-lc")
        .arg(cmd);

    Ok(command)
}

fn build_shell_command(cmd: &str, host_cwd: &Path) -> Result<Command> {
    #[cfg(target_os = "linux")]
    {
        build_linux_sandbox_command(cmd, host_cwd)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let mut command = if cfg!(target_os = "windows") {
            let shell_bin = if command_in_path("pwsh.exe").is_some() || command_in_path("pwsh").is_some() {
                "pwsh"
            } else if command_in_path("powershell.exe").is_some() {
                "powershell.exe"
            } else {
                return Err(anyhow::anyhow!("PowerShell(pwsh 또는 powershell.exe)을 찾을 수 없습니다. Windows 환경에서 ExecShell을 실행할 수 없습니다."));
            };
            let mut c = Command::new(shell_bin);
            c.arg("-Command").arg(cmd);
            c
        } else {
            let mut c = Command::new("sh");
            c.arg("-c").arg(cmd);
            c
        };
        command.current_dir(host_cwd);
        Ok(command)
    }
}

/// [v0.1.0-beta.18] Phase 9-C: 스트리밍 모드 셸 실행.
/// tx가 Some일 경우 각 출력 라인을 ToolOutputChunk로 비동기 전송.
/// tx가 None이면 버퍼 모드(기존 동작)와 동일.
pub(crate) async fn execute_shell_streaming(
    cmd: &str,
    cwd: Option<&str>,
    tx: Option<tokio::sync::mpsc::Sender<crate::app::event_loop::Event>>,
) -> Result<ToolResult> {
    let host_cwd = resolve_shell_cwd(cwd)?;
    let mut command = build_shell_command(cmd, &host_cwd)?;

    // [v0.1.0-beta.18] 스트리밍을 위해 stdout/stderr를 파이프로 연결
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());

    let timeout_result =
        tokio::time::timeout(std::time::Duration::from_secs(SHELL_TIMEOUT_SECS), async {
            let mut child = command.spawn()?;

            let stdout_handle = child.stdout.take();
            let stderr_handle = child.stderr.take();

            // stdout 비동기 라인 읽기
            let tx_clone = tx.clone();
            let stdout_task = tokio::spawn(async move {
                let mut lines = String::new();
                if let Some(stdout) = stdout_handle {
                    let mut reader = BufReader::new(stdout).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        // ToolOutputChunk 이벤트 전송 (tx가 있는 경우)
                        if let Some(ref tx) = tx_clone {
                            let _ = tx
                                .send(crate::app::event_loop::Event::Action(
                                    crate::app::action::Action::ToolOutputChunk(format!(
                                        "[stdout] {}",
                                        line
                                    )),
                                ))
                                .await;
                        }
                        lines.push_str(&line);
                        lines.push('\n');
                    }
                }
                lines
            });

            // stderr 비동기 라인 읽기
            let tx_clone2 = tx;
            let stderr_task = tokio::spawn(async move {
                let mut lines = String::new();
                if let Some(stderr) = stderr_handle {
                    let mut reader = BufReader::new(stderr).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        if let Some(ref tx) = tx_clone2 {
                            let _ = tx
                                .send(crate::app::event_loop::Event::Action(
                                    crate::app::action::Action::ToolOutputChunk(format!(
                                        "[stderr] {}",
                                        line
                                    )),
                                ))
                                .await;
                        }
                        lines.push_str(&line);
                        lines.push('\n');
                    }
                }
                lines
            });

            // 프로세스 완료 대기 + 출력 수집
            let status = child.wait().await?;
            let stdout_buf = stdout_task.await.unwrap_or_default();
            let stderr_buf = stderr_task.await.unwrap_or_default();

            let exit_code = status.code().unwrap_or(1);

            Ok::<ToolResult, anyhow::Error>(ToolResult {
                tool_name: "ExecShell".to_string(),
                stdout: stdout_buf,
                stderr: stderr_buf,
                exit_code,
                is_error: !status.success(),
                tool_call_id: None,
            })
        })
        .await;

    match timeout_result {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(e)) => Ok(ToolResult {
            tool_name: "ExecShell".to_string(),
            stdout: String::new(),
            stderr: format!("프로세스 실행 실패: {}", e),
            exit_code: 1,
            is_error: true,
            tool_call_id: None,
        }),
        Err(_) => Ok(ToolResult {
            tool_name: "ExecShell".to_string(),
            stdout: String::new(),
            stderr: format!("프로세스 타임아웃 ({}초 초과).", SHELL_TIMEOUT_SECS),
            exit_code: 1,
            is_error: true,
            tool_call_id: None,
        }),
    }
}

// ==========================================
// Phase 13: Agentic Autonomy Tool Registry
// ==========================================

use crate::domain::error::ToolError;
use crate::domain::permissions::{PermissionResult, ShellPolicy};
use crate::domain::settings::PersistedSettings;
use crate::tools::registry::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{Value, json};

pub struct ExecShellTool;

#[async_trait]
impl Tool for ExecShellTool {
    fn name(&self) -> &'static str {
        "ExecShell"
    }

    fn description(&self) -> &'static str {
        "Executes a shell command."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "ExecShell",
                "description": "Execute a shell command. Use this to run tests, build, or interact with the OS.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": { "type": "string" },
                        "cwd": { "type": "string", "description": "Current working directory. Default: '.'" },
                        "safe_to_auto_run": { "type": "boolean", "description": "Set to true if command is read-only or safe" }
                    },
                    "required": ["command"]
                }
            }
        })
    }

    fn check_permission(&self, args: &Value, settings: &PersistedSettings) -> PermissionResult {
        let command = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
        let safe_to_auto_run = args
            .get("safe_to_auto_run")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if command.trim().is_empty() {
            return PermissionResult::Deny("빈 명령은 실행할 수 없습니다.".to_string());
        }

        // 위험 명령어 블랙리스트 처리 (임시 하드코딩, 나중에 PermissionEngine 로직 재사용)
        let lower = command.to_lowercase();
        let blocked = [
            "sudo ",
            "rm -rf",
            "rm -fr",
            "chmod 777",
            "chmod -R 777",
            "mkfs",
            "dd if=",
            "> /dev/",
            ":(){ :|:&",
            "shutdown",
            "reboot",
            "init 0",
            "init 6",
            "format c:",
            "del /f /s",
        ];
        if blocked.iter().any(|pattern| lower.contains(pattern)) {
            return PermissionResult::Deny(format!(
                "위험 명령어로 차단됨: '{}'",
                command.chars().take(60).collect::<String>()
            ));
        }

        match settings.shell_policy {
            ShellPolicy::Deny => {
                PermissionResult::Deny("Shell execution is disabled by policy.".to_string())
            }
            ShellPolicy::Ask => PermissionResult::Ask,
            ShellPolicy::SafeOnly => {
                let parts: Vec<&str> = command.split_whitespace().collect();
                if parts.is_empty() {
                    return PermissionResult::Deny(
                        "빈 명령은 안전하지 않음으로 분류됩니다.".to_string(),
                    );
                }

                let is_custom_safe = settings
                    .safe_commands
                    .as_ref()
                    .is_some_and(|c| c.iter().any(|cmd| cmd == parts[0]));
                let os = std::env::consts::OS;
                let is_builtin_safe = if os == "windows" {
                    ["dir", "echo", "date", "cd", "type", "find"].contains(&parts[0])
                } else {
                    [
                        "ls", "pwd", "date", "echo", "cat", "grep", "df", "free", "uname",
                    ]
                    .contains(&parts[0])
                };

                if safe_to_auto_run || is_custom_safe || is_builtin_safe {
                    PermissionResult::Allow
                } else {
                    PermissionResult::Deny(format!(
                        "Command '{}' is blocked in SafeOnly mode.",
                        command
                    ))
                }
            }
        }
    }

    fn format_detail(&self, args: &Value) -> String {
        let command = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
        let cwd_str = args.get("cwd").and_then(|v| v.as_str()).unwrap_or(".");
        format!("승인 대기 (y/n) — 명령: '{}' (cwd: {})", command, cwd_str)
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext<'_>) -> Result<ToolResult, ToolError> {
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let cwd = args
            .get("cwd")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // TODO: action_tx가 필요한 streaming은 execute 인자로 어떻게 넘길지 나중에 개선
        execute_shell(&command, cwd.as_deref())
            .await
            .map_err(|e| ToolError::ExecutionFailure(e.to_string()))
    }
}
