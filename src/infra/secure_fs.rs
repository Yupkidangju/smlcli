use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub fn ensure_private_dir(path: &Path) -> std::io::Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(std::io::Error::other(format!(
                    "private directory가 symlink이거나 directory가 아닙니다: {}",
                    path.display()
                )));
            }
            verify_owner(&metadata, path)?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir_all(path)?;
        }
        Err(error) => return Err(error),
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

#[cfg(unix)]
fn verify_owner(metadata: &std::fs::Metadata, path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::MetadataExt;
    if metadata.uid() != unsafe { libc::geteuid() } {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!(
                "현재 사용자가 소유하지 않은 private path입니다: {}",
                path.display()
            ),
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn verify_owner(_metadata: &std::fs::Metadata, _path: &Path) -> std::io::Result<()> {
    Ok(())
}

fn validate_existing_regular(path: &Path) -> std::io::Result<Option<std::fs::Metadata>> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(std::io::Error::other(format!(
                    "private file이 symlink이거나 regular file이 아닙니다: {}",
                    path.display()
                )));
            }
            verify_owner(&metadata, path)?;
            Ok(Some(metadata))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(unix)]
fn private_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    use std::os::unix::fs::OpenOptionsExt;
    options
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    options
}

#[cfg(not(unix))]
fn private_options() -> OpenOptions {
    OpenOptions::new()
}

pub fn create_private_new(path: &Path) -> std::io::Result<File> {
    if let Some(parent) = path.parent() {
        ensure_private_dir(parent)?;
    }
    let mut options = private_options();
    options.write(true).create_new(true);
    let file = options.open(path)?;
    set_private_file_permissions(path)?;
    Ok(file)
}

pub fn open_private_lock(path: &Path) -> std::io::Result<File> {
    if let Some(parent) = path.parent() {
        ensure_private_dir(parent)?;
    }
    validate_existing_regular(path)?;
    let mut options = private_options();
    options.read(true).write(true).create(true);
    let file = options.open(path)?;
    set_private_file_permissions(path)?;
    Ok(file)
}

pub fn open_private_append(path: &Path) -> std::io::Result<File> {
    if let Some(parent) = path.parent() {
        ensure_private_dir(parent)?;
    }
    validate_existing_regular(path)?;
    let mut options = private_options();
    options.append(true).create(true);
    let file = options.open(path)?;
    set_private_file_permissions(path)?;
    Ok(file)
}

pub fn read_private_limited(path: &Path, max_bytes: usize) -> std::io::Result<Vec<u8>> {
    let metadata = validate_existing_regular(path)?.ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, path.display().to_string())
    })?;
    if metadata.len() > max_bytes as u64 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("private file이 {max_bytes} bytes 제한을 초과했습니다"),
        ));
    }
    let mut options = private_options();
    options.read(true);
    let mut file = options.open(path)?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    std::io::Read::by_ref(&mut file)
        .take(max_bytes.saturating_add(1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > max_bytes {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "private file read limit 초과",
        ));
    }
    set_private_file_permissions(path)?;
    Ok(bytes)
}

pub fn atomic_write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "private file parent 없음")
    })?;
    ensure_private_dir(parent)?;
    validate_existing_regular(path)?;

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("data");
    let mut temporary: Option<(PathBuf, File)> = None;
    for _ in 0..16 {
        let temp_path = parent.join(format!(".{file_name}.smlcli-{}.tmp", uuid::Uuid::new_v4()));
        match create_private_new(&temp_path) {
            Ok(file) => {
                temporary = Some((temp_path, file));
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    let (temp_path, mut file) = temporary.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "unique private temp 없음",
        )
    })?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temp_path, path)?;
        set_private_file_permissions(path)?;
        if let Ok(directory) = File::open(parent) {
            let _ = directory.sync_all();
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temp_path);
    }
    result
}

#[cfg(unix)]
pub fn set_private_file_permissions(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
pub fn set_private_file_permissions(_path: &Path) -> std::io::Result<()> {
    Ok(())
}
