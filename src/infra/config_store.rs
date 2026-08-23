// [v0.1.0-beta.15] YAML → TOML 교체.
// [v0.1.0-beta.19] tokio::fs를 사용한 비동기 I/O 전환.
// [v0.1.0-beta.20] thiserror 기반 ConfigError 연동.
//   anyhow::Result 반환을 유지하되 내부에서 ConfigError를 사용하여
//   에러 유형을 구조화. 향후 UI에서 에러 종류별 분기 처리 가능.

use crate::domain::error::{ConfigError, SmlError};
use crate::domain::settings::PersistedSettings;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::fs;

static CONFIG_SAVE_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
static NEXT_CONFIG_REVISION: AtomicU64 = AtomicU64::new(1);
static COMMITTED_CONFIG_REVISION: AtomicU64 = AtomicU64::new(0);

/// 설정 디렉토리: ~/.smlcli/
/// ~/.smlcli 디렉토리 경로 반환
pub fn get_config_dir() -> PathBuf {
    #[cfg(test)]
    {
        static TEST_CONFIG_ROOT: std::sync::OnceLock<tempfile::TempDir> =
            std::sync::OnceLock::new();
        TEST_CONFIG_ROOT
            .get_or_init(|| tempfile::tempdir().expect("isolated test config root"))
            .path()
            .to_path_buf()
    }
    #[cfg(not(test))]
    {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".smlcli")
    }
}

/// 설정 파일 전체 경로: ~/.smlcli/config.toml
pub(crate) fn config_path() -> PathBuf {
    get_config_dir().join("config.toml")
}

/// 설정을 TOML 형식으로 디스크에 비동기 저장.
pub async fn save_config(settings: &PersistedSettings) -> Result<(), SmlError> {
    save_config_to_path(settings, &config_path()).await
}

pub(crate) async fn save_config_to_path(
    settings: &PersistedSettings,
    path: &Path,
) -> Result<(), SmlError> {
    let mut clean_settings = settings.clone();
    clean_settings.trusted_workspaces.retain(|r| r.remember);

    let toml_str = toml::to_string(&clean_settings).map_err(|e| {
        SmlError::Config(ConfigError::ParseFailure(format!(
            "설정 TOML 직렬화 실패: {}",
            e
        )))
    })?;

    let revision = NEXT_CONFIG_REVISION.fetch_add(1, Ordering::SeqCst);
    let _writer_guard = CONFIG_SAVE_MUTEX.lock().await;
    if revision < COMMITTED_CONFIG_REVISION.load(Ordering::SeqCst) {
        return Ok(());
    }
    let path = path.to_path_buf();
    let lock_path = path.with_extension("toml.lock");
    tokio::task::spawn_blocking(move || {
        use fs2::FileExt;
        let lock_file = crate::infra::secure_fs::open_private_lock(&lock_path)?;
        lock_file.lock_exclusive()?;
        crate::infra::secure_fs::atomic_write_private(&path, toml_str.as_bytes())
    })
    .await
    .map_err(|error| SmlError::InfraError(format!("config writer join 실패: {error}")))?
    .map_err(|error| {
        if error.kind() == std::io::ErrorKind::StorageFull {
            SmlError::InfraError(
                "디스크 용량이 부족하여 설정 저장을 중단합니다. 기존 설정은 보존됩니다."
                    .to_string(),
            )
        } else {
            SmlError::Config(ConfigError::ParseFailure(format!(
                "private atomic config 저장 실패: {error}"
            )))
        }
    })?;
    COMMITTED_CONFIG_REVISION.store(revision, Ordering::SeqCst);
    Ok(())
}

/// 디스크에서 TOML 설정 비동기 로드.
/// [v0.1.0-beta.20] 내부에서 ConfigError를 사용하여 에러를 구조화.
pub async fn load_config() -> Result<Option<PersistedSettings>, SmlError> {
    let path = config_path();
    let mut settings_opt = load_config_from_path(&path).await?;

    // [v2.2.0] Phase 30: Config Schema Versioning Auto-Migration
    // [v2.3.0] Phase 31: Backup & Rollback safety net
    if let Some(ref mut settings) = settings_opt
        && settings.migrate()
    {
        let bak_path = path.with_extension("toml.bak");
        // 1. Backup
        let backup_success = fs::copy(&path, &bak_path).await.is_ok();

        // 2. Save
        if let Err(e) = save_config(settings).await {
            // 3. Rollback on failure
            if backup_success {
                let _ = fs::rename(&bak_path, &path).await;
            }
            return Err(SmlError::Config(ConfigError::ParseFailure(format!(
                "설정 마이그레이션 실패 (롤백됨): {}",
                e
            ))));
        }
    }

    Ok(settings_opt)
}

/// [v0.1.0-beta.26] 경로 지정형 설정 로더.
/// 실제 앱은 기본 config.toml 경로를 사용하고, 테스트는 임시 파일 경로를 직접 주입한다.
pub(crate) async fn load_config_from_path(
    path: &Path,
) -> Result<Option<PersistedSettings>, SmlError> {
    let path = path.to_path_buf();

    if !path.exists() {
        return Ok(None);
    }

    // [v0.1.0-beta.21] I/O 에러 종류를 정확히 분류.
    // 파일 미존재(NotFound)와 권한 거부/기타 I/O 실패를 구분하여
    // 사용자에게 정확한 진단 메시지를 전달한다.
    let read_path = path.clone();
    let bytes = tokio::task::spawn_blocking(move || {
        crate::infra::secure_fs::read_private_limited(&read_path, 4 * 1024 * 1024)
    })
    .await
    .map_err(|error| SmlError::InfraError(format!("config reader join 실패: {error}")))?
    .map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => SmlError::Config(ConfigError::NotFound),
        std::io::ErrorKind::PermissionDenied => SmlError::Config(ConfigError::ParseFailure(
            format!("파일 접근 권한 없음: {}", path.display()),
        )),
        _ => SmlError::IoError(e),
    })?;
    let content = String::from_utf8(bytes).map_err(|error| {
        SmlError::Config(ConfigError::ParseFailure(format!(
            "config.toml이 UTF-8이 아닙니다: {error}"
        )))
    })?;

    let settings: PersistedSettings = toml::from_str(&content)
        .map_err(|e| SmlError::Config(ConfigError::ParseFailure(e.to_string())))?;

    Ok(Some(settings))
}
