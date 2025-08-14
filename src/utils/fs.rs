use crate::error::{FlowError, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn find_repository_root(start_path: &Path) -> Option<PathBuf> {
    let mut current_path = start_path;
    
    loop {
        let flow_dir = current_path.join(".flowversion");
        if flow_dir.exists() && flow_dir.is_dir() {
            return Some(current_path.to_path_buf());
        }
        
        match current_path.parent() {
            Some(parent) => current_path = parent,
            None => return None,
        }
    }
}

pub fn is_ignored(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    
    // Ignore FlowVersion directory
    if path_str.contains(".flowversion") {
        return true;
    }
    
    // Ignore common files and directories
    let ignore_patterns = [
        ".git",
        ".gitignore",
        "node_modules",
        "target",
        ".DS_Store",
        "Thumbs.db",
        ".tmp",
        ".temp",
    ];
    
    for pattern in &ignore_patterns {
        if path_str.contains(pattern) {
            return true;
        }
    }
    
    false
}

pub fn list_tracked_files(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    for entry in WalkDir::new(repo_root)
        .into_iter()
        .filter_entry(|e| !is_ignored(e.path()))
    {
        let entry = entry.map_err(|e| FlowError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Walk directory error: {}", e)
        )))?;
        
        if entry.file_type().is_file() {
            if let Ok(relative_path) = entry.path().strip_prefix(repo_root) {
                files.push(relative_path.to_path_buf());
            }
        }
    }
    
    Ok(files)
}

pub fn normalize_path(path: &Path) -> PathBuf {
    // Convert to forward slashes for consistency across platforms
    let path_str = path.to_string_lossy().replace('\\', "/");
    PathBuf::from(path_str)
}

pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_find_repository_root() {
        let temp_dir = TempDir::new().unwrap();
        let repo_root = temp_dir.path();
        let flow_dir = repo_root.join(".flowversion");
        std::fs::create_dir(&flow_dir).unwrap();
        
        let sub_dir = repo_root.join("sub").join("directory");
        std::fs::create_dir_all(&sub_dir).unwrap();
        
        let found_root = find_repository_root(&sub_dir);
        assert_eq!(found_root, Some(repo_root.to_path_buf()));
    }
    
    #[test]
    fn test_find_repository_root_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("no-repo");
        std::fs::create_dir(&path).unwrap();
        
        let found_root = find_repository_root(&path);
        assert_eq!(found_root, None);
    }
    
    #[test]
    fn test_is_ignored() {
        assert!(is_ignored(Path::new(".flowversion/objects")));
        assert!(is_ignored(Path::new("node_modules/package")));
        assert!(is_ignored(Path::new("target/debug/app")));
        assert!(is_ignored(Path::new(".git/config")));
        
        assert!(!is_ignored(Path::new("src/main.rs")));
        assert!(!is_ignored(Path::new("README.md")));
    }
    
    #[test]
    fn test_normalize_path() {
        let windows_path = Path::new("src\\main.rs");
        let normalized = normalize_path(windows_path);
        
        assert_eq!(normalized.to_string_lossy(), "src/main.rs");
    }
}