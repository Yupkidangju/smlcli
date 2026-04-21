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
        assert_eq!(found.canonicalize().unwrap_or(found.clone()), root.canonicalize().unwrap_or(root.to_path_buf()));
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
        assert_eq!(found.canonicalize().unwrap_or(found.clone()), root.canonicalize().unwrap_or(root.to_path_buf()));
    }

    #[test]
    fn test_find_workspace_root_fallback() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("a").join("b");
        fs::create_dir_all(&subdir).unwrap();

        let found = find_workspace_root(&subdir);
        assert_eq!(found.canonicalize().unwrap_or(found.clone()), subdir.canonicalize().unwrap_or(subdir.clone()));
    }
}
