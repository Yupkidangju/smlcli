// [v0.1.0-beta.18] Phase 9-C: Shell stdout/stderr 실시간 스트리밍 구현.
// 이전: command.output()으로 전체 완료 후 일괄 수집 (30초 타임아웃)
// 현재: stdout/stderr를 라인 단위로 비동기 스트리밍하여 ToolOutputChunk 이벤트 발행 가능.
//       action_tx가 없는 경우(기존 호출)는 버퍼 모드로 동작하여 하위 호환 유지.
// spec.md §9 Phase 9-C 참조.

use crate::domain::tool_result::ToolResult;
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// [v0.1.0-beta.18] 셸 실행 타임아웃 (초).
const SHELL_TIMEOUT_SECS: u64 = 30;

/// 셸 명령어를 실행하고 stdout/stderr를 수집하여 ToolResult로 반환.
/// action_tx가 Some이면 각 라인을 ToolOutputChunk 이벤트로 실시간 전송.
pub(crate) async fn execute_shell(
    cmd: &str,
    cwd: Option<&str>,
) -> Result<ToolResult> {
    execute_shell_streaming(cmd, cwd, None).await
}

/// [v0.1.0-beta.18] Phase 9-C: 스트리밍 모드 셸 실행.
/// tx가 Some일 경우 각 출력 라인을 ToolOutputChunk로 비동기 전송.
/// tx가 None이면 버퍼 모드(기존 동작)와 동일.
pub(crate) async fn execute_shell_streaming(
    cmd: &str,
    cwd: Option<&str>,
    tx: Option<tokio::sync::mpsc::Sender<crate::app::event_loop::Event>>,
) -> Result<ToolResult> {
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

    // [v0.1.0-beta.18] 스트리밍을 위해 stdout/stderr를 파이프로 연결
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());

    let timeout_result = tokio::time::timeout(
        std::time::Duration::from_secs(SHELL_TIMEOUT_SECS),
        async {
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
                                    crate::app::action::Action::ToolOutputChunk(
                                        format!("[stdout] {}", line),
                                    ),
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
                                    crate::app::action::Action::ToolOutputChunk(
                                        format!("[stderr] {}", line),
                                    ),
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
            })
        },
    )
    .await;

    match timeout_result {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(e)) => Ok(ToolResult {
            tool_name: "ExecShell".to_string(),
            stdout: String::new(),
            stderr: format!("프로세스 실행 실패: {}", e),
            exit_code: 1,
            is_error: true,
        }),
        Err(_) => Ok(ToolResult {
            tool_name: "ExecShell".to_string(),
            stdout: String::new(),
            stderr: format!("프로세스 타임아웃 ({}초 초과).", SHELL_TIMEOUT_SECS),
            exit_code: 1,
            is_error: true,
        }),
    }
}
