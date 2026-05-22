use std::env;
use std::path::{Path, PathBuf};

/// Finds the workspace root directory by traversing upwards from the given starting path.
/// It looks for `.git` or `Cargo.toml`.
/// Returns the found root path, or the original starting path if not found.
pub fn find_workspace_root(start_path: impl AsRef<Path>) -> PathBuf {
    let mut current = start_path.as_ref().to_path_buf();

    // Canonicalize to avoid issues with "." and ".."
    if let Ok(canon) = current.canonicalize() {
        current = canon;
    }

    let original = current.clone();

    loop {
        if current.join(".git").exists() || current.join("Cargo.toml").exists() {
            return current;
        }

        if !current.pop() {
            break;
        }
    }

    original
}

/// Helper to get the workspace root based on the current working directory.
pub fn get_current_workspace_root() -> PathBuf {
    if let Ok(cwd) = env::current_dir() {
        find_workspace_root(cwd)
    } else {
        PathBuf::from(".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_find_workspace_root_with_git() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // Create .git dir
        fs::create_dir(root.join(".git")).unwrap();

        let subdir = root.join("src").join("module");
        fs::create_dir_all(&subdir).unwrap();

        let found = find_workspace_root(&subdir);
        assert_eq!(
            found.canonicalize().unwrap_or(found.clone()),
            root.canonicalize().unwrap_or(root.to_path_buf())
        );
    }

    #[test]
    fn test_find_workspace_root_with_cargo_toml() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // Create Cargo.toml
        fs::write(root.join("Cargo.toml"), "").unwrap();

        let subdir = root.join("src");
        fs::create_dir_all(&subdir).unwrap();

        let found = find_workspace_root(&subdir);
        assert_eq!(
            found.canonicalize().unwrap_or(found.clone()),
            root.canonicalize().unwrap_or(root.to_path_buf())
        );
    }

    #[test]
    fn test_find_workspace_root_fallback() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("a").join("b");
        fs::create_dir_all(&subdir).unwrap();

        let found = find_workspace_root(&subdir);
        let found_canon = found.canonicalize().unwrap_or(found.clone());

        // [v3.8.1] /tmp/.git 등 상위 디렉토리에 뜻하지 않게 .git이나 Cargo.toml이 존재하여
        // find_workspace_root가 더 상위에서 멈추는 시스템 환경 노이즈 방어 로직 적용
        let mut expected = subdir.canonicalize().unwrap_or(subdir.clone());
        let mut matched = false;
        loop {
            if found_canon == expected {
                matched = true;
                break;
            }
            if expected.join(".git").exists() || expected.join("Cargo.toml").exists() {
                // 상위에 실제로 .git이나 Cargo.toml이 존재하므로 find_workspace_root가 여기서 멈춘 것이 맞음
                if found_canon == expected {
                    matched = true;
                }
                break;
            }
            if !expected.pop() {
                break;
            }
        }

        assert!(
            matched,
            "Expected found path ({:?}) to be either subdir ({:?}) or one of its parents containing .git/Cargo.toml",
            found_canon, subdir
        );
    }
}
