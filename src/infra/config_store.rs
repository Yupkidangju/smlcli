// [v0.1.0-beta.15] YAML(serde_yml, unsound) → TOML(이미 의존성 존재)으로 교체.
// 경로: ~/.smlcli/config.toml
// 설정값은 평문 TOML로 저장. API 키만 encrypted_keys 맵에 암호화된 형태로 보관.
// [Low] config.toml에도 chmod 600 적용하여 다른 사용자 접근 차단.

use crate::domain::settings::PersistedSettings;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// 설정 디렉토리: ~/.smlcli/
fn get_config_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".smlcli")
}

/// 설정 파일 전체 경로: ~/.smlcli/config.toml
fn config_path() -> PathBuf {
    get_config_dir().join("config.toml")
}

/// 설정을 TOML 형식으로 디스크에 저장.
/// API 키는 이미 encrypted_keys에 암호화된 상태이므로 추가 암호화 불필요.
/// Unix 환경에서는 config.toml에도 chmod 600을 설정하여 소유자만 접근 가능.
pub fn save_config(settings: &PersistedSettings) -> Result<()> {
    let toml_str = toml::to_string(settings).context("설정 TOML 직렬화 실패")?;

    let config_dir = get_config_dir();
    std::fs::create_dir_all(&config_dir).context("~/.smlcli 디렉토리 생성 실패")?;

    let path = config_path();
    std::fs::write(&path, toml_str).context("config.toml 저장 실패")?;

    // Unix: 암호화 키를 포함하므로 소유자만 읽기/쓰기 가능하도록 권한 설정
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .context("config.toml 권한(600) 설정 실패")?;
    }

    Ok(())
}

/// 디스크에서 TOML 설정 로드.
/// 파일이 없으면 None 반환 (초기 설정 필요 → Wizard 실행).
pub fn load_config() -> Result<Option<PersistedSettings>> {
    let path = config_path();

    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path).context("config.toml 읽기 실패")?;

    let settings: PersistedSettings =
        toml::from_str(&content).context("config.toml 파싱 실패 (형식 오류 또는 손상)")?;

    Ok(Some(settings))
}
