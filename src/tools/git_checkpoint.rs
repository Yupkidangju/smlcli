// [v0.1.0-beta.23] Phase 13: Agentic Autonomy
// 에이전트 자율 모드(Run) 중 파괴적인 도구 호출 전에 자동으로 상태를 커밋하고,
// 실패 시 롤백(rollback)하기 위한 Git 체크포인트 모듈입니다.

use anyhow::Result;
use std::process::Command;

/// 현재 저장소가 git으로 관리되고 있는지 확인합니다.
pub fn is_git_repo(cwd: &str) -> bool {
    let status = Command::new("git")
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
        .arg("status")
        .arg("--porcelain")
        .current_dir(cwd)
        .output();
    
    match status {
        Ok(out) => !out.stdout.is_empty(),
        Err(_) => false,
    }
}

/// 현재 워킹 트리가 깨끗한지 확인하여 롤백이 안전한지 반환합니다.
/// 사용자 WIP가 있는 경우 롤백 시 날아갈 위험이 있으므로 false를 반환합니다.
pub fn create_checkpoint(cwd: &str, _tool_name: &str) -> Result<bool> {
    if !is_git_repo(cwd) {
        return Ok(false); // Git 저장소가 아니면 스킵
    }
    
    if has_uncommitted_changes(cwd) {
        // WIP가 있으면 롤백 시 날아갈 위험이 있으므로 체크포인트 생성 생략
        return Ok(false);
    }

    // 변경사항이 없으므로(HEAD 상태와 동일) 롤백이 안전함. 강제 커밋 제거.
    Ok(true)
}

/// 가장 최근의 커밋(HEAD) 상태로 추적 중인(tracked) 파일만 되돌립니다.
/// [v0.1.0-beta.23] git clean -fd 완전 제거: untracked 사용자 파일 보호.
/// git reset --hard의 종료 코드를 반드시 검사하여 롤백 실패 시 에러를 전파합니다.
pub fn rollback_checkpoint(cwd: &str) -> Result<()> {
    if !is_git_repo(cwd) {
        return Ok(()); // Git 저장소가 아니면 스킵
    }

    // git reset --hard HEAD — tracked 파일만 HEAD 상태로 복원
    let status = Command::new("git")
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
