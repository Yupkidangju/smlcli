// [v0.1.0-beta.23] Phase 13: Agentic Autonomy
// 에이전트 자율 모드(Run) 중 파괴적인 도구 호출 전에 자동으로 상태를 커밋하고,
// 실패 시 롤백(rollback)하기 위한 Git 체크포인트 모듈입니다.

use crate::domain::error::ToolError;
use crate::domain::permissions::PermissionResult;
use crate::domain::settings::PersistedSettings;
use crate::domain::tool_result::ToolResult;
use crate::tools::registry::{Tool, ToolContext};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::process::Command;

pub struct GitCheckpointTool;

#[async_trait]
impl Tool for GitCheckpointTool {
    fn name(&self) -> &'static str {
        "GitCheckpoint"
    }

    fn description(&self) -> &'static str {
        "Creates a git checkpoint for the current workspace state. Used automatically before destructive changes to allow safe rollbacks."
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "GitCheckpoint",
                "description": "Create a git checkpoint for the current workspace state.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "tool_name": { "type": "string", "description": "Name of the tool that triggered the checkpoint." }
                    }
                }
            }
        })
    }

    fn check_permission(&self, _args: &Value, _settings: &PersistedSettings) -> PermissionResult {
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string());

        if !is_git_repo(&cwd) {
            return PermissionResult::Deny("현재 디렉토리가 Git 저장소가 아닙니다.".to_string());
        }
        PermissionResult::Allow
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext<'_>) -> Result<ToolResult, ToolError> {
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string());

        let tool_name = args
            .get("tool_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Manual");

        match create_checkpoint(&cwd, tool_name) {
            Ok(true) => Ok(ToolResult {
                tool_name: "GitCheckpoint".to_string(),
                stdout: "Git 체크포인트가 생성되었습니다.".to_string(),
                stderr: String::new(),
                exit_code: 0,
                is_error: false,
                tool_call_id: None,
                is_truncated: false,
                original_size_bytes: None,
                affected_paths: vec![],
            }),
            Ok(false) => Ok(ToolResult {
                tool_name: "GitCheckpoint".to_string(),
                stdout: "체크포인트 생성이 스킵되었습니다 (변경사항 존재 등).".to_string(),
                stderr: String::new(),
                exit_code: 0,
                is_error: false,
                tool_call_id: None,
                is_truncated: false,
                original_size_bytes: None,
                affected_paths: vec![],
            }),
            Err(e) => Err(ToolError::ExecutionFailure(e.to_string())),
        }
    }
}

/// [v2.0.0] Phase 28: 최대 유지할 체크포인트 개수
pub const MAX_GIT_CHECKPOINTS: usize = 50;

/// 오래된 체크포인트 refs를 삭제하여 저장소 용량을 관리합니다.
pub fn prune_checkpoints(cwd: &str) -> Result<()> {
    let out = Command::new("git")
        .stdin(std::process::Stdio::null())
        .args([
            "for-each-ref",
            "--sort=-committerdate",
            "--format=%(refname)",
            "refs/smlcli/checkpoints/",
        ])
        .current_dir(cwd)
        .output()?;

    let refs = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = refs.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() > MAX_GIT_CHECKPOINTS {
        for ref_to_delete in lines.iter().skip(MAX_GIT_CHECKPOINTS) {
            let _ = Command::new("git")
                .stdin(std::process::Stdio::null())
                .args(["update-ref", "-d", ref_to_delete])
                .current_dir(cwd)
                .status();
        }
    }
    Ok(())
}

/// 현재 저장소가 git으로 관리되고 있는지 확인합니다.
pub fn is_git_repo(cwd: &str) -> bool {
    let status = Command::new("git")
        .stdin(std::process::Stdio::null())
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .current_dir(cwd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match status {
        Ok(s) => s.success(),
        Err(_) => false,
    }
}

/// 현재 워킹 트리에 변경된 파일이 있는지 확인합니다.
pub fn has_uncommitted_changes(cwd: &str) -> bool {
    let status = Command::new("git")
        .stdin(std::process::Stdio::null())
        .arg("status")
        .arg("--porcelain")
        .current_dir(cwd)
        .output();

    match status {
        Ok(out) => !out.stdout.is_empty(),
        Err(_) => false,
    }
}

/// 현재 진행 중인 Git 충돌 해결(Merge/Rebase/Cherry-pick) 상태인지 확인합니다.
pub fn has_merge_conflict(cwd: &str) -> bool {
    let out = Command::new("git")
        .stdin(std::process::Stdio::null())
        .arg("rev-parse")
        .arg("--git-dir")
        .current_dir(cwd)
        .output();

    if let Ok(output) = out {
        let path_str = String::from_utf8_lossy(&output.stdout);
        let git_dir_path = std::path::Path::new(path_str.trim());
        let git_dir = if git_dir_path.is_absolute() {
            git_dir_path.to_path_buf()
        } else {
            std::path::Path::new(cwd).join(git_dir_path)
        };

        return git_dir.join("MERGE_HEAD").exists()
            || git_dir.join("REBASE_HEAD").exists()
            || git_dir.join("CHERRY_PICK_HEAD").exists();
    }

    false
}

/// 현재 워킹 트리가 깨끗한지 확인하여 롤백이 안전한지 반환합니다.
/// 사용자 WIP가 있는 경우 롤백 시 날아갈 위험이 있으므로 false를 반환합니다.
pub fn create_checkpoint(cwd: &str, tool_name: &str) -> Result<bool> {
    if !is_git_repo(cwd) {
        return Ok(false); // Git 저장소가 아니면 스킵
    }

    // [v2.1.0] Phase 29: 초기화되지 않은 저장소(커밋 0개) 대응
    let has_commits = Command::new("git")
        .stdin(std::process::Stdio::null())
        .args(["rev-list", "-n", "1", "--all"])
        .current_dir(cwd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !has_commits {
        return Ok(false); // 커밋이 없으면 레퍼런스 생성이 불가하므로 스킵
    }

    if has_merge_conflict(cwd) {
        return Err(anyhow::anyhow!(
            "현재 Git 충돌 해결 중이므로 자동 체크포인트를 생성할 수 없습니다"
        ));
    }

    if has_uncommitted_changes(cwd) {
        // WIP가 있으면 롤백 시 날아갈 위험이 있으므로 체크포인트 생성 생략
        return Ok(false);
    }

    // [v2.0.0] Phase 28: 체크포인트 ref 생성 및 Prune
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // 도구 이름에서 특수문자를 제거하여 ref_name에 적합하게 만듦
    let safe_tool_name = tool_name.replace(|c: char| !c.is_alphanumeric(), "_");
    let ref_name = format!("refs/smlcli/checkpoints/{}_{}", timestamp, safe_tool_name);

    let status = Command::new("git")
        .stdin(std::process::Stdio::null())
        .args(["update-ref", &ref_name, "HEAD"])
        .current_dir(cwd)
        .status();

    if status.is_ok_and(|s| s.success()) {
        let _ = prune_checkpoints(cwd);
    }

    Ok(true)
}

/// 가장 최근의 커밋(HEAD) 상태로 추적 중인(tracked) 파일만 되돌립니다.
/// [v0.1.0-beta.23] git clean -fd 완전 제거: untracked 사용자 파일 보호.
/// git reset --hard의 종료 코드를 반드시 검사하여 롤백 실패 시 에러를 전파합니다.
pub fn rollback_checkpoint(cwd: &str) -> Result<()> {
    if !is_git_repo(cwd) {
        return Ok(()); // Git 저장소가 아니면 스킵
    }

    // [v2.1.0] Phase 29: 커밋 0개인 상태에서는 HEAD가 없으므로 리셋 불가
    let has_commits = Command::new("git")
        .stdin(std::process::Stdio::null())
        .args(["rev-list", "-n", "1", "--all"])
        .current_dir(cwd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !has_commits {
        return Ok(());
    }

    // git reset --hard HEAD — tracked 파일만 HEAD 상태로 복원
    let status = Command::new("git")
        .stdin(std::process::Stdio::null())
        .arg("reset")
        .arg("--hard")
        .arg("HEAD")
        .current_dir(cwd)
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "git reset --hard HEAD 실패 (exit code: {:?}). 롤백을 중단합니다.",
            status.code()
        ));
    }

    // [v0.1.0-beta.23] git clean -fd 삭제됨.
    // 사유: untracked 파일(사용자 WIP, 새 파일 등)을 무조건 삭제하여 데이터 유실 위험이 있었음.
    // 삭제 버전: v0.1.0-beta.23 (감사 보고서 H-1 대응)

    Ok(())
}
