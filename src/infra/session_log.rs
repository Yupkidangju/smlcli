// [v0.1.0-beta.18] Phase 10: 세션 영속성 — JSONL 기반 대화 로그 저장/복원.
// 각 메시지를 한 줄의 JSON 레코드로 기록하여 라인 단위 추가(append-only)를 보장.
// 세션 시작 시 `~/.smlcli/sessions/` 디렉토리에 `session_{timestamp}.jsonl` 파일 생성.
// 복원 시 JSONL 파일을 순차 파싱하여 SessionState.messages를 재구성.
// spec.md §9 Phase 10 (세션 영속성) 참조.
// 외부 의존성 최소화: chrono 없이 std::time + UNIX 타임스탬프 사용.

use crate::providers::types::{ChatMessage, Role};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// [v0.1.0-beta.18] JSONL 레코드 포맷.
/// 각 줄은 이 구조체의 JSON 직렬화 결과.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SessionRecord {
    /// 레코드 타임스탬프 (UNIX epoch seconds)
    ts: u64,
    /// 메시지 역할
    role: Role,
    /// 메시지 내용
    content: String,
    /// pinned 플래그
    #[serde(default)]
    pinned: bool,
}

/// [v0.1.0-beta.18] 세션 로그 관리자.
/// append-only 방식으로 대화 메시지를 JSONL 파일에 기록하고,
/// 기존 세션 파일로부터 메시지를 복원하는 기능 제공.
pub struct SessionLogger {
    /// 현재 세션의 JSONL 파일 경로
    file_path: PathBuf,
}

impl SessionLogger {
    /// 새 세션 로그 파일을 생성.
    /// `~/.smlcli/sessions/session_{timestamp}.jsonl` 경로에 파일 생성.
    pub fn new_session() -> Result<Self> {
        let sessions_dir = Self::sessions_dir()?;
        fs::create_dir_all(&sessions_dir)
            .context("세션 디렉토리 생성 실패")?;

        let ts = Self::unix_timestamp();
        let file_name = format!("session_{}.jsonl", ts);
        let file_path = sessions_dir.join(file_name);

        // 빈 파일 생성
        fs::File::create(&file_path)
            .context("세션 로그 파일 생성 실패")?;

        Ok(Self { file_path })
    }

    /// 기존 세션 파일로부터 로거를 생성 (복원용).
    pub fn from_file(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            anyhow::bail!("세션 파일이 존재하지 않습니다: {}", path.display());
        }
        Ok(Self { file_path: path })
    }

    /// 메시지를 JSONL 파일에 추가.
    /// append 모드로 열어 한 줄 기록.
    pub fn append_message(&self, msg: &ChatMessage) -> Result<()> {
        let record = SessionRecord {
            ts: Self::unix_timestamp(),
            role: msg.role.clone(),
            content: msg.content.clone(),
            pinned: msg.pinned,
        };

        let json_line = serde_json::to_string(&record)
            .context("세션 레코드 직렬화 실패")?;

        let mut file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.file_path)
            .context("세션 로그 파일 열기 실패")?;

        writeln!(file, "{}", json_line)
            .context("세션 로그 쓰기 실패")?;

        Ok(())
    }

    /// JSONL 파일에서 메시지 목록을 복원.
    /// 파싱 실패한 라인은 건너뛰고 에러 카운트를 반환.
    pub fn restore_messages(&self) -> Result<(Vec<ChatMessage>, usize)> {
        let file = fs::File::open(&self.file_path)
            .context("세션 로그 파일 열기 실패")?;
        let reader = std::io::BufReader::new(file);

        let mut messages = Vec::new();
        let mut errors = 0;

        for line in reader.lines() {
            let Ok(line_str) = line else {
                errors += 1;
                continue;
            };

            // 빈 줄 건너뛰기
            let trimmed = line_str.trim();
            if trimmed.is_empty() {
                continue;
            }

            match serde_json::from_str::<SessionRecord>(trimmed) {
                Ok(record) => {
                    messages.push(ChatMessage {
                        role: record.role,
                        content: record.content,
                        pinned: record.pinned,
                    });
                }
                Err(_) => {
                    errors += 1;
                }
            }
        }

        Ok((messages, errors))
    }

    /// 사용 가능한 세션 파일 목록을 최신순으로 반환.
    /// 각 항목은 (파일명, 파일 크기, 레코드 수) 튜플.
    pub fn list_sessions() -> Result<Vec<(String, u64, usize)>> {
        let sessions_dir = Self::sessions_dir()?;
        if !sessions_dir.exists() {
            return Ok(vec![]);
        }

        let mut sessions = Vec::new();
        for entry in fs::read_dir(&sessions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                let name = path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                // 레코드 수: 줄 수로 추정
                let lines = fs::read_to_string(&path)
                    .map(|c| c.lines().filter(|l| !l.trim().is_empty()).count())
                    .unwrap_or(0);
                sessions.push((name, size, lines));
            }
        }

        // 파일명 역순 정렬 (최신순)
        sessions.sort_by(|a, b| b.0.cmp(&a.0));
        Ok(sessions)
    }

    /// 현재 로그 파일 경로 반환.
    pub fn path(&self) -> &Path {
        &self.file_path
    }

    /// 세션 디렉토리 경로: `~/.smlcli/sessions/`
    fn sessions_dir() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .context("홈 디렉토리를 찾을 수 없습니다")?;
        Ok(home.join(".smlcli").join("sessions"))
    }

    /// 현재 UNIX 타임스탬프 (초 단위) 반환.
    fn unix_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}
