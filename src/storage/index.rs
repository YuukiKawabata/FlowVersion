use serde::{Deserialize, Serialize};
use crate::error::{FlowError, Result};
use crate::core::objects::Hash;
use std::collections::HashMap;
use std::path::Path;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub hash: Hash,
    pub size: u64,
    pub modified_time: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    entries: RwLock<HashMap<String, IndexEntry>>,
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

impl Index {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }
    
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        
        let content = std::fs::read_to_string(path)?;
        
        if content.trim().is_empty() {
            return Ok(Self::new());
        }
        
        let entries: HashMap<String, IndexEntry> = serde_json::from_str(&content)?;
        
        Ok(Self {
            entries: RwLock::new(entries),
        })
    }
    
    pub fn save(&self, path: &Path) -> Result<()> {
        let entries = self.entries.try_read()
            .map_err(|_| FlowError::InternalError("Failed to acquire read lock".to_string()))?;
        
        let content = serde_json::to_string_pretty(&*entries)?;
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(path, content)?;
        Ok(())
    }
    
    pub async fn add_file(&self, path: String, hash: Hash, size: u64) -> Result<()> {
        let mut entries = self.entries.write().await;
        
        let entry = IndexEntry {
            hash,
            size,
            modified_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        entries.insert(path, entry);
        Ok(())
    }
    
    pub async fn remove_file(&self, path: &str) -> Result<()> {
        let mut entries = self.entries.write().await;
        entries.remove(path);
        Ok(())
    }
    
    pub async fn get_file(&self, path: &str) -> Result<Option<IndexEntry>> {
        let entries = self.entries.read().await;
        Ok(entries.get(path).cloned())
    }
    
    pub async fn get_staged_files(&self) -> Result<HashMap<String, IndexEntry>> {
        let entries = self.entries.read().await;
        Ok(entries.clone())
    }
    
    pub async fn is_file_staged(&self, path: &str) -> Result<bool> {
        let entries = self.entries.read().await;
        Ok(entries.contains_key(path))
    }
    
    pub async fn clear(&self) -> Result<()> {
        let mut entries = self.entries.write().await;
        entries.clear();
        Ok(())
    }
    
    pub async fn get_status(&self) -> Result<IndexStatus> {
        let entries = self.entries.read().await;
        
        Ok(IndexStatus {
            staged_count: entries.len(),
            total_size: entries.values().map(|e| e.size).sum(),
        })
    }
}

#[derive(Debug)]
pub struct IndexStatus {
    pub staged_count: usize,
    pub total_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_index_operations() {
        let index = Index::new();
        let hash = Hash::new(b"test content");
        
        // Add file
        index.add_file("test.txt".to_string(), hash.clone(), 100).await.unwrap();
        
        // Check if file is staged
        assert!(index.is_file_staged("test.txt").await.unwrap());
        
        // Get file entry
        let entry = index.get_file("test.txt").await.unwrap();
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().hash, hash);
        
        // Get all staged files
        let staged = index.get_staged_files().await.unwrap();
        assert_eq!(staged.len(), 1);
        assert!(staged.contains_key("test.txt"));
        
        // Remove file
        index.remove_file("test.txt").await.unwrap();
        assert!(!index.is_file_staged("test.txt").await.unwrap());
    }
    
    #[tokio::test]
    async fn test_index_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("index.json");
        
        let index = Index::new();
        let hash = Hash::new(b"test content");
        
        index.add_file("test.txt".to_string(), hash.clone(), 100).await.unwrap();
        index.save(&index_path).unwrap();
        
        let loaded_index = Index::load(&index_path).unwrap();
        let entry = loaded_index.get_file("test.txt").await.unwrap();
        
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().hash, hash);
    }
    
    #[tokio::test]
    async fn test_index_status() {
        let index = Index::new();
        
        let status = index.get_status().await.unwrap();
        assert_eq!(status.staged_count, 0);
        assert_eq!(status.total_size, 0);
        
        index.add_file("file1.txt".to_string(), Hash::new(b"content1"), 50).await.unwrap();
        index.add_file("file2.txt".to_string(), Hash::new(b"content2"), 75).await.unwrap();
        
        let status = index.get_status().await.unwrap();
        assert_eq!(status.staged_count, 2);
        assert_eq!(status.total_size, 125);
    }
}