// [v0.1.0-beta.18] Phase 10: 세션 영속성 — JSONL 기반 대화 로그 저장/복원.
// [v0.1.0-beta.19] tokio::fs를 사용한 비동기 I/O 전환.
// [v0.1.0-beta.20] 동기 API 복원: from_file, append_message(동기), restore_messages.
//   비동기 전환 과정에서 삭제된 세션 복원/테스트용 동기 API를 재공급.
//   비동기 append_message_async는 런타임 호출 경로에서 사용하고,
//   동기 append_message/from_file/restore_messages는 테스트 및 복원 시나리오에서 사용한다.

use crate::domain::error::SmlError;
use crate::providers::types::ChatMessage;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;

pub struct SessionLogger {
    pub file_path: PathBuf,
    writer: Mutex<BufWriter<File>>,
}

impl std::fmt::Debug for SessionLogger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionLogger")
            .field("file_path", &self.file_path)
            .finish()
    }
}

impl Drop for SessionLogger {
    fn drop(&mut self) {
        if let Ok(mut w) = self.writer.lock() {
            let _ = w.flush();
            if let Ok(file) = w.get_ref().try_clone() {
                let _ = file.sync_all();
            }
        }
    }
}

impl SessionLogger {
    /// 새 세션 로그 파일 생성: session_{timestamp}.jsonl
    pub fn new_session() -> Result<Self, SmlError> {
        let log_dir = Self::get_log_dir()?;
        if !log_dir.exists() {
            std::fs::create_dir_all(&log_dir).map_err(|e| SmlError::InfraError(format!("세션 로그 디렉토리 생성 실패: {}", e)))?;
        }

        let timestamp = Self::unix_timestamp();
        let file_path = log_dir.join(format!("session_{}.jsonl", timestamp));

        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .map_err(|e| SmlError::InfraError(format!("세션 로그 파일 열기 실패: {}", e)))?;

        Ok(Self {
            file_path,
            writer: Mutex::new(BufWriter::new(file)),
        })
    }

    /// [v0.1.0-beta.20] 기존 JSONL 파일로부터 로거를 생성. 세션 복원용.
    /// 파일이 존재하지 않으면 에러를 반환한다.
    pub fn from_file(path: PathBuf) -> Result<Self, SmlError> {
        if !path.exists() {
            return Err(SmlError::InfraError(format!("세션 로그 파일이 존재하지 않습니다: {:?}", path)));
        }
        
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| SmlError::InfraError(format!("세션 로그 파일 열기 실패: {}", e)))?;

        Ok(Self {
            file_path: path,
            writer: Mutex::new(BufWriter::new(file)),
        })
    }

    /// [v0.1.0-beta.20] 동기 메시지 추가: 테스트 및 단순 호출 경로용.
    /// JSONL 한 줄을 동기 I/O로 파일에 append 한다.
    pub fn append_message(&self, message: &ChatMessage) -> Result<(), SmlError> {
        let json_line = serde_json::to_string(message).map_err(|e| SmlError::InfraError(format!("메시지 직렬화 실패: {}", e)))?;
        
        let mut w = self.writer.lock().map_err(|_| SmlError::InfraError("로거 락 획득 실패".into()))?;
        writeln!(w, "{}", json_line).map_err(|e| SmlError::InfraError(format!("세션 로그 쓰기 실패: {}", e)))?;
        // [v1.0.0] BufWriter 플러시 타이밍 결함 해소 (비정상 종료 시 로그 유실 방지)
        w.flush().map_err(|e| SmlError::InfraError(format!("세션 로그 flush 실패: {}", e)))?;
        Ok(())
    }

    /// 메시지를 JSONL 한 줄로 추가 저장. (비동기 — 런타임 호출 경로용)
    pub async fn append_message_async(&self, message: &ChatMessage) -> Result<(), SmlError> {
        // [v1.0.0] BufWriter를 사용하므로 비동기 호출에서도 동기 메서드를 사용하거나 tokio의 blocking으로 처리합니다.
        let json_line = serde_json::to_string(message).map_err(|e| SmlError::InfraError(format!("메시지 직렬화 실패: {}", e)))?;
        
        let mut w = self.writer.lock().map_err(|_| SmlError::InfraError("로거 락 획득 실패".into()))?;
        writeln!(w, "{}", json_line).map_err(|e| SmlError::InfraError(format!("세션 로그 쓰기 실패: {}", e)))?;
        w.flush().map_err(|e| SmlError::InfraError(format!("세션 로그 flush 실패: {}", e)))?;
        Ok(())
    }

    /// [v0.1.0-beta.20] JSONL 파일에서 메시지를 복원.
    /// 반환값: (성공적으로 파싱된 메시지 목록, 파싱 실패 라인 수)
    /// 손상된 라인은 건너뛰고 나머지를 최대한 복원한다.
    pub fn restore_messages(&self) -> Result<(Vec<ChatMessage>, usize), SmlError> {
        // 읽기 전에 먼저 flush
        if let Ok(mut w) = self.writer.lock() {
            let _ = w.flush();
        }

        let content =
            std::fs::read_to_string(&self.file_path).map_err(|e| SmlError::InfraError(format!("세션 로그 파일 읽기 실패: {}", e)))?;

        let mut messages = Vec::new();
        let mut errors = 0usize;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            match serde_json::from_str::<ChatMessage>(trimmed) {
                Ok(msg) => messages.push(msg),
                Err(_) => errors += 1,
            }
        }

        Ok((messages, errors))
    }

    fn get_log_dir() -> Result<PathBuf, SmlError> {
        let home = dirs::home_dir().ok_or_else(|| SmlError::InfraError("홈 디렉토리를 찾을 수 없습니다".into()))?;
        Ok(home.join(".smlcli").join("sessions"))
    }

    fn unix_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// 저장된 모든 세션 로그 파일 목록과 정보를 비동기로 조회.
    /// 반환값: Vec<(파일명, 파일크기, 메시지수)>
    pub async fn list_sessions() -> Result<Vec<(String, u64, usize)>, SmlError> {
        let log_dir = Self::get_log_dir()?;
        if !log_dir.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        let mut entries = fs::read_dir(log_dir)
            .await
            .map_err(|e| SmlError::InfraError(format!("세션 디렉토리 읽기 실패: {}", e)))?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| SmlError::InfraError(format!("디렉토리 항목 읽기 실패: {}", e)))? {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let metadata = entry.metadata().await.map_err(|e| SmlError::InfraError(format!("메타데이터 읽기 실패: {}", e)))?;
                let size = metadata.len();

                // 메시지 수(라인 수) 계산
                let content = fs::read_to_string(&path).await.unwrap_or_default();
                let line_count = content.lines().count();

                sessions.push((name, size, line_count));
            }
        }

        // 최신 순 정렬
        sessions.sort_by(|a, b| b.0.cmp(&a.0));
        Ok(sessions)
    }
}
