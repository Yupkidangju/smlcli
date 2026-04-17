// [v0.1.0-beta.15] YAML → TOML 교체.
// [v0.1.0-beta.19] tokio::fs를 사용한 비동기 I/O 전환.
// [v0.1.0-beta.20] thiserror 기반 ConfigError 연동.
//   anyhow::Result 반환을 유지하되 내부에서 ConfigError를 사용하여
//   에러 유형을 구조화. 향후 UI에서 에러 종류별 분기 처리 가능.

use crate::domain::error::ConfigError;
use crate::domain::settings::PersistedSettings;
use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;

/// 설정 디렉토리: ~/.smlcli/
fn get_config_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".smlcli")
}

/// 설정 파일 전체 경로: ~/.smlcli/config.toml
fn config_path() -> PathBuf {
    get_config_dir().join("config.toml")
}

/// 설정을 TOML 형식으로 디스크에 비동기 저장.
pub async fn save_config(settings: &PersistedSettings) -> Result<()> {
    let toml_str = toml::to_string(settings).context("설정 TOML 직렬화 실패")?;

    let config_dir = get_config_dir();
    fs::create_dir_all(&config_dir)
        .await
        .context("~/.smlcli 디렉토리 생성 실패")?;

    let path = config_path();
    fs::write(&path, toml_str)
        .await
        .context("config.toml 저장 실패")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path).await?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms)
            .await
            .context("config.toml 권한 설정 실패")?;
    }

    Ok(())
}

/// 디스크에서 TOML 설정 비동기 로드.
/// [v0.1.0-beta.20] 내부에서 ConfigError를 사용하여 에러를 구조화.
pub async fn load_config() -> Result<Option<PersistedSettings>> {
    let path = config_path();

    if !path.exists() {
        return Ok(None);
    }

    // [v0.1.0-beta.21] I/O 에러 종류를 정확히 분류.
    // 파일 미존재(NotFound)와 권한 거부/기타 I/O 실패를 구분하여
    // 사용자에게 정확한 진단 메시지를 전달한다.
    let content = fs::read_to_string(&path)
        .await
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => ConfigError::NotFound,
            std::io::ErrorKind::PermissionDenied => {
                ConfigError::ParseFailure(format!("파일 접근 권한 없음: {}", path.display()))
            }
            _ => ConfigError::ParseFailure(format!("파일 읽기 실패: {}", e)),
        })
        .context("config.toml 읽기 실패")?;

    let settings: PersistedSettings = toml::from_str(&content)
        .map_err(|e| ConfigError::ParseFailure(e.to_string()))
        .context("config.toml 파싱 실패")?;

    Ok(Some(settings))
}
