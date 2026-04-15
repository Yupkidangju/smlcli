// [v0.1.0-beta.14] 아키텍처 변경: 바이너리 암호화 설정 → YAML 평문 설정.
// 경로: ~/.smlcli/config.yaml
// 설정값은 평문 YAML로 저장. API 키만 encrypted_keys 맵에 암호화된 형태로 보관.
// 이전: settings.enc (ChaCha20Poly1305 전체 암호화, master_key 필요)
// 변경: config.yaml (YAML, 설정 평문 + API 키만 암호화)

use crate::domain::settings::PersistedSettings;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// 설정 디렉토리: ~/.smlcli/
fn get_config_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".smlcli")
}

/// 설정 파일 전체 경로: ~/.smlcli/config.yaml
fn config_path() -> PathBuf {
    get_config_dir().join("config.yaml")
}

/// 설정을 YAML 형식으로 디스크에 저장.
/// master_key는 더 이상 필요하지 않음 (API 키는 이미 encrypted_keys에 암호화된 상태).
pub fn save_config(settings: &PersistedSettings) -> Result<()> {
    let yaml_str = serde_yml::to_string(settings).context("설정 YAML 직렬화 실패")?;

    let config_dir = get_config_dir();
    std::fs::create_dir_all(&config_dir).context("~/.smlcli 디렉토리 생성 실패")?;

    std::fs::write(config_path(), yaml_str).context("config.yaml 저장 실패")?;

    Ok(())
}

/// 디스크에서 YAML 설정 로드.
/// 파일이 없으면 None 반환 (초기 설정 필요 → Wizard 실행).
/// master_key는 더 이상 필요하지 않음.
pub fn load_config() -> Result<Option<PersistedSettings>> {
    let path = config_path();

    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path).context("config.yaml 읽기 실패")?;

    let settings: PersistedSettings =
        serde_yml::from_str(&content).context("config.yaml 파싱 실패 (형식 오류 또는 손상)")?;

    Ok(Some(settings))
}
