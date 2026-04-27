// [v3.0.0] Phase 40: Git-Native Integration
// 파일 변경 내역을 자동으로 커밋하고 히스토리를 관리하는 인프라 엔진입니다.

// [v3.4.0] Phase 44 Task D-2: TECH-DEBT 정리 완료. 파일 레벨 allow(dead_code) 제거.

use anyhow::Result;
use std::process::Command;

/// 체크포인트 엔트리 (TUI 히스토리 표시용)
// [v3.7.0] ref_name, timestamp, tool_name, files_changed 필드는
// Inspector Git 탭 렌더링에서 활성화 예정.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CheckpointEntry {
    pub ref_name: String,
    pub timestamp: u64,
    pub tool_name: String,
    pub files_changed: Vec<String>,
    pub commit_hash: String,
    pub author: String,
    pub message: String,
}

pub struct GitEngine;

impl GitEngine {
    /// 변경된 파일을 stage하고 자동 커밋 생성
    /// 커밋 메시지: "{prefix}{tool_name}: {파일 경로 요약}"
    /// [v2.5.3] 감사 MEDIUM-2: files가 비어있으면 WIP 혼입 방지를 위해 즉시 skip.
    /// 기존 git add -u fallback은 사용자 WIP를 포함할 위험이 있어 제거.
    pub fn auto_commit(cwd: &str, tool_name: &str, files: &[&str], prefix: &str) -> Result<String> {
        // 1) 파일 목록이 비어있으면 skip (WIP 보호)
        if files.is_empty() {
            return Err(anyhow::anyhow!(
                "auto_commit skip: 변경 파일 목록이 비어있습니다. WIP 보호를 위해 커밋하지 않습니다."
            ));
        } else {
            for file in files {
                // [v2.5.1] 감사 MEDIUM-2: 파일별 git add 실패 시 에러를 전파.
                // 존재하지 않는 파일(git이 관리하지 않는)은 경고만 하고 계속 진행.
                let add_status = Command::new("git")
                    .args(["add", file])
                    .current_dir(cwd)
                    .status()?;

                if !add_status.success() {
                    return Err(anyhow::anyhow!(
                        "git add '{}' failed. 파일이 존재하지 않거나 stage할 수 없습니다.",
                        file
                    ));
                }
            }
        }

        // 2) 변경사항 있는지 확인 (staged changes)
        let diff_status = Command::new("git")
            .args(["diff", "--cached", "--quiet"])
            .current_dir(cwd)
            .status()?;

        if diff_status.success() {
            // 변경된 내용 없음
            return Ok("No changes to commit".to_string());
        }

        // 3) 커밋 메시지 생성
        let files_summary = if files.is_empty() {
            "various files".to_string()
        } else if files.len() == 1 {
            files[0].to_string()
        } else {
            format!("{} and {} other files", files[0], files.len() - 1)
        };

        let message = format!("{}{}: {}", prefix, tool_name, files_summary);

        // 4) 커밋 실행
        let commit_status = Command::new("git")
            .args(["commit", "-m", &message])
            .current_dir(cwd)
            .status()?;

        if !commit_status.success() {
            return Err(anyhow::anyhow!("git commit failed"));
        }

        Ok(message)
    }

    /// [v2.5.2] 감사 MEDIUM-1 수정: 메시지 매칭 + 해시 consumed 추적 방식의 undo.
    /// 동작 원리:
    /// 1) HEAD가 smlcli 자동 커밋이면 HEAD를 직접 revert.
    /// 2) HEAD가 Revert/사용자 커밋이면 git log에서 최근 50건을 탐색하여:
    ///    a) Revert "X" 커밋의 원본 메시지 X를 수집.
    ///    b) smlcli 커밋 중 메시지가 X와 일치하는 것을 consumed(이미 reverted)로 표시.
    ///    c) 아직 consumed되지 않은 가장 최근 smlcli 커밋을 해시로 revert.
    ///
    /// 한계: 동일 메시지의 커밋이 3건 이상이면 순서 의존적 매칭이 부정확할 수 있음.
    /// 이는 auto_commit이 항상 파일명을 포함하는 유니크 메시지를 생성하므로 실무상 문제 없음.
    pub fn undo_last(cwd: &str, prefix: &str) -> Result<String> {
        // HEAD 커밋 메시지 확인
        let out = Command::new("git")
            .args(["log", "-1", "--pretty=%s"])
            .current_dir(cwd)
            .output()?;

        if !out.status.success() {
            return Err(anyhow::anyhow!("Failed to read git log"));
        }

        let msg = String::from_utf8_lossy(&out.stdout).trim().to_string();

        // 1) HEAD가 직접 smlcli 자동 커밋이면 바로 revert
        if msg.starts_with(prefix) {
            return Self::revert_head(cwd, &msg);
        }

        // 2) HEAD가 Revert 커밋이거나 사용자 커밋이면,
        //    git log에서 아직 revert되지 않은 가장 최근 smlcli 커밋을 탐색
        //    해시 기반으로 추적: Revert 커밋의 parent가 원본 커밋의 hash
        let log_out = Command::new("git")
            .args([
                "log",
                "--pretty=format:%H|%P|%s",
                "-50", // 최근 50건만 탐색
            ])
            .current_dir(cwd)
            .output()?;

        if !log_out.status.success() {
            return Err(anyhow::anyhow!("Failed to read git log for undo search"));
        }

        let stdout = String::from_utf8_lossy(&log_out.stdout);
        // [v3.3.1] 감사 MEDIUM-4 정합화: 실제 매칭 전략은 "메시지 매칭 + 해시 consumed 추적".
        // Revert "X" 커밋이 있으면 원본 메시지 X를 수집하고,
        // smlcli 커밋 중 메시지가 X인 것을 순서대로 consumed 표시하여 중복 undo 방지.
        // 해시는 consumed 추적과 git revert 대상 지정에만 사용.
        let mut reverted_hashes: Vec<String> = Vec::new();
        let mut smlcli_commits: Vec<(String, String)> = Vec::new(); // (hash, message)
        let mut revert_msgs: Vec<String> = Vec::new(); // Revert에서 추출한 원본 메시지

        // 1차 수집: 모든 커밋을 순회하며 분류
        for line in stdout.lines() {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() != 3 {
                continue;
            }
            let hash = parts[0];
            let commit_msg = parts[2];

            // Revert 커밋인지 확인하고, 원본 메시지를 추출
            if let Some(inner) = commit_msg
                .strip_prefix("Revert \"")
                .and_then(|s| s.strip_suffix('"'))
            {
                revert_msgs.push(inner.to_string());
                continue;
            }

            // smlcli 자동 커밋 수집
            if commit_msg.starts_with(prefix) {
                smlcli_commits.push((hash.to_string(), commit_msg.to_string()));
            }
        }

        // 2차 매칭: revert 메시지와 smlcli 커밋을 메시지 기반으로 매칭 후 해시를 consumed 표시.
        // 각 revert 메시지에 대해, 가장 최근(위쪽) smlcli 커밋 중 메시지가 일치하는 것을 consumed
        for rev_msg in &revert_msgs {
            for (hash, msg) in &smlcli_commits {
                if msg == rev_msg && !reverted_hashes.contains(hash) {
                    reverted_hashes.push(hash.clone());
                    break;
                }
            }
        }

        // 3차: 아직 revert되지 않은 가장 최근 smlcli 커밋을 찾음
        for (hash, msg) in &smlcli_commits {
            if !reverted_hashes.contains(hash) {
                let revert_status = Command::new("git")
                    .args(["revert", "--no-edit", hash])
                    .current_dir(cwd)
                    .status()?;

                if !revert_status.success() {
                    return Err(anyhow::anyhow!(
                        "git revert failed. 작업 트리에 충돌이 있을 수 있습니다."
                    ));
                }

                return Ok(format!("Undo 성공: {}", msg));
            }
        }

        Err(anyhow::anyhow!(
            "되돌릴 수 있는 smlcli 자동 커밋을 찾지 못했습니다."
        ))
    }

    /// HEAD 커밋을 직접 revert하는 내부 헬퍼
    fn revert_head(cwd: &str, msg: &str) -> Result<String> {
        let revert_status = Command::new("git")
            .args(["revert", "--no-edit", "HEAD"])
            .current_dir(cwd)
            .status()?;

        if !revert_status.success() {
            return Err(anyhow::anyhow!(
                "git revert failed. 작업 트리에 충돌이 있을 수 있습니다."
            ));
        }

        Ok(format!("Undo 성공: {}", msg))
    }

    /// [v2.5.1] 감사 MEDIUM-1 수정: prefix 인자를 받아 smlcli 생성 커밋만 필터링.
    /// prefix가 비어있으면 전체 히스토리를 반환 (호환성 유지).
    pub fn list_history(cwd: &str, prefix: &str, limit: usize) -> Result<Vec<CheckpointEntry>> {
        let mut args = vec![
            "log".to_string(),
            format!("-{}", limit),
            "--pretty=format:%H|%ct|%an|%s".to_string(),
        ];

        // prefix가 주어지면 git log --grep으로 커밋 메시지 필터링
        if !prefix.is_empty() {
            args.push(format!("--grep=^{}", prefix));
        }

        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let out = Command::new("git")
            .args(&arg_refs)
            .current_dir(cwd)
            .output()?;

        if !out.status.success() {
            return Ok(vec![]); // Git repo가 아니거나 히스토리 없음
        }

        let mut entries = Vec::new();
        let stdout = String::from_utf8_lossy(&out.stdout);

        for line in stdout.lines() {
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() == 4 {
                let hash = parts[0].to_string();
                let timestamp = parts[1].parse::<u64>().unwrap_or(0);
                let author = parts[2].to_string();
                let msg = parts[3].to_string();

                entries.push(CheckpointEntry {
                    ref_name: hash.clone(),
                    timestamp,
                    tool_name: "git".to_string(),
                    files_changed: vec![],
                    commit_hash: hash,
                    author,
                    message: msg,
                });
            }
        }

        Ok(entries)
    }
}
