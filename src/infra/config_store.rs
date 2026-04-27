// [v0.1.0-beta.15] YAML → TOML 교체.
// [v0.1.0-beta.19] tokio::fs를 사용한 비동기 I/O 전환.
// [v0.1.0-beta.20] thiserror 기반 ConfigError 연동.
//   anyhow::Result 반환을 유지하되 내부에서 ConfigError를 사용하여
//   에러 유형을 구조화. 향후 UI에서 에러 종류별 분기 처리 가능.

use crate::domain::error::{ConfigError, SmlError};
use crate::domain::settings::PersistedSettings;
use std::path::{Path, PathBuf};
use tokio::fs;

/// 설정 디렉토리: ~/.smlcli/
/// ~/.smlcli 디렉토리 경로 반환
pub fn get_config_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".smlcli")
}

/// 설정 파일 전체 경로: ~/.smlcli/config.toml
pub(crate) fn config_path() -> PathBuf {
    get_config_dir().join("config.toml")
}

/// 설정을 TOML 형식으로 디스크에 비동기 저장.
pub async fn save_config(settings: &PersistedSettings) -> Result<(), SmlError> {
    // [v0.1.0-beta.26] 메모리 전용 레코드(remember == false) 필터링 후 저장
    let mut clean_settings = settings.clone();
    clean_settings.trusted_workspaces.retain(|r| r.remember);

    let toml_str = toml::to_string(&clean_settings).map_err(|e| {
        SmlError::Config(ConfigError::ParseFailure(format!(
            "설정 TOML 직렬화 실패: {}",
            e
        )))
    })?;

    let config_dir = get_config_dir();
    fs::create_dir_all(&config_dir).await.map_err(|e| {
        SmlError::Config(ConfigError::ParseFailure(format!(
            "~/.smlcli 디렉토리 생성 실패: {}",
            e
        )))
    })?;

    let path = config_path();

    // [v2.0.0] Phase 28: File Locking을 통한 동시 쓰기 경합 방지
    let lock_path = path.with_extension("toml.lock");
    let lock_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&lock_path)
        .map_err(|e| {
            SmlError::Config(ConfigError::ParseFailure(format!(
                "config.toml.lock 파일 열기 실패: {}",
                e
            )))
        })?;

    use fs2::FileExt;
    lock_file.lock_exclusive().map_err(|e| {
        SmlError::Config(ConfigError::ParseFailure(format!(
            "설정 파일 락 획득 실패: {}",
            e
        )))
    })?;

    let tmp_path = path.with_extension("toml.tmp");

    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        options.mode(0o600);
    }

    let mut file = options.open(&tmp_path).await.map_err(|e| {
        SmlError::Config(ConfigError::ParseFailure(format!(
            "config.toml.tmp 임시 파일 생성 실패: {}",
            e
        )))
    })?;

    use tokio::io::AsyncWriteExt;
    file.write_all(toml_str.as_bytes()).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::StorageFull {
            SmlError::InfraError(
                "디스크 용량이 부족하여 설정 저장을 중단합니다. 기존 설정은 보존됩니다.".into(),
            )
        } else {
            SmlError::Config(ConfigError::ParseFailure(format!(
                "config.toml.tmp 저장 실패: {}",
                e
            )))
        }
    })?;

    // [v1.4.0] fsync 호출로 디스크 기록 보장
    file.sync_all().await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::StorageFull {
            SmlError::InfraError(
                "디스크 용량이 부족하여 설정 저장을 중단합니다. 기존 설정은 보존됩니다.".into(),
            )
        } else {
            SmlError::Config(ConfigError::ParseFailure(format!(
                "config.toml.tmp 동기화 실패: {}",
                e
            )))
        }
    })?;

    // [v1.4.0] 원본 파일로 원자적 덮어쓰기(rename)
    fs::rename(&tmp_path, &path).await.map_err(|e| {
        SmlError::Config(ConfigError::ParseFailure(format!(
            "config.toml 원자적 교체 실패: {}",
            e
        )))
    })?;

    Ok(())
}

/// [v2.2.0] Phase 30: Atomic 쓰기 실패 시 남겨진 .tmp 파일 일괄 정리
pub async fn cleanup_tmp_files() {
    let dir = get_config_dir();
    if let Ok(mut entries) = fs::read_dir(&dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_file()
                && let Some(ext) = path.extension()
                && ext == "tmp"
            {
                let _ = fs::remove_file(path).await;
            }
        }
    }
}

/// 디스크에서 TOML 설정 비동기 로드.
/// [v0.1.0-beta.20] 내부에서 ConfigError를 사용하여 에러를 구조화.
pub async fn load_config() -> Result<Option<PersistedSettings>, SmlError> {
    cleanup_tmp_files().await;

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
    let content = fs::read_to_string(&path)
        .await
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => SmlError::Config(ConfigError::NotFound),
            std::io::ErrorKind::PermissionDenied => SmlError::Config(ConfigError::ParseFailure(
                format!("파일 접근 권한 없음: {}", path.display()),
            )),
            _ => SmlError::IoError(e),
        })?;

    let settings: PersistedSettings = toml::from_str(&content)
        .map_err(|e| SmlError::Config(ConfigError::ParseFailure(e.to_string())))?;

    Ok(Some(settings))
}
