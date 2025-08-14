use crate::error::{FlowError, Result};
use crate::core::{objects::Hash, commit::FlowCommit, objects::{CommitId, BlobObject}};
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct ObjectStore {
    objects_path: PathBuf,
}

impl ObjectStore {
    pub fn new(objects_path: &Path) -> Self {
        Self {
            objects_path: objects_path.to_path_buf(),
        }
    }
    
    pub async fn store_blob(&self, hash: &Hash, content: Vec<u8>) -> Result<()> {
        let blob_path = self.get_blob_path(hash);
        
        if blob_path.exists() {
            // Blob already exists
            return Ok(());
        }
        
        // Create directory if it doesn't exist
        if let Some(parent) = blob_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        // Store blob
        let blob = BlobObject::new(content);
        let serialized = serde_json::to_vec(&blob)?;
        
        fs::write(&blob_path, serialized).await?;
        
        Ok(())
    }
    
    pub async fn load_blob(&self, hash: &Hash) -> Result<BlobObject> {
        let blob_path = self.get_blob_path(hash);
        
        if !blob_path.exists() {
            return Err(FlowError::ObjectNotFound(hash.to_string()));
        }
        
        let content = fs::read(&blob_path).await?;
        let blob: BlobObject = serde_json::from_slice(&content)?;
        
        Ok(blob)
    }
    
    pub async fn store_commit(&self, commit: &FlowCommit) -> Result<()> {
        let commit_path = self.get_commit_path(&commit.id);
        
        if commit_path.exists() {
            return Err(FlowError::ObjectAlreadyExists(commit.id.to_string()));
        }
        
        // Create directory if it doesn't exist
        if let Some(parent) = commit_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        // Store commit
        let serialized = commit.to_bytes()?;
        fs::write(&commit_path, serialized).await?;
        
        Ok(())
    }
    
    pub async fn load_commit(&self, commit_id: &CommitId) -> Result<FlowCommit> {
        let commit_path = self.get_commit_path(commit_id);
        
        if !commit_path.exists() {
            return Err(FlowError::ObjectNotFound(commit_id.to_string()));
        }
        
        let content = fs::read(&commit_path).await?;
        let commit = FlowCommit::from_bytes(&content)?;
        
        Ok(commit)
    }
    
    pub async fn commit_exists(&self, commit_id: &CommitId) -> bool {
        let commit_path = self.get_commit_path(commit_id);
        commit_path.exists()
    }
    
    pub async fn blob_exists(&self, hash: &Hash) -> bool {
        let blob_path = self.get_blob_path(hash);
        blob_path.exists()
    }
    
    pub async fn list_commits(&self) -> Result<Vec<CommitId>> {
        let commits_dir = self.objects_path.join("commits");
        
        if !commits_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut commits = Vec::new();
        let mut entries = fs::read_dir(&commits_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.ends_with(".json") {
                        let commit_id_str = file_name.trim_end_matches(".json");
                        if let Ok(uuid) = uuid::Uuid::parse_str(commit_id_str) {
                            commits.push(CommitId::from_uuid(uuid));
                        }
                    }
                }
            }
        }
        
        Ok(commits)
    }
    
    pub async fn get_stats(&self) -> Result<ObjectStoreStats> {
        let mut blob_count = 0;
        let mut commit_count = 0;
        let mut total_size = 0;
        
        // Count blobs
        let blobs_dir = self.objects_path.join("blobs");
        if blobs_dir.exists() {
            let mut entries = fs::read_dir(&blobs_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                if entry.path().is_file() {
                    blob_count += 1;
                    total_size += entry.metadata().await?.len();
                }
            }
        }
        
        // Count commits
        let commits_dir = self.objects_path.join("commits");
        if commits_dir.exists() {
            let mut entries = fs::read_dir(&commits_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                if entry.path().is_file() {
                    commit_count += 1;
                    total_size += entry.metadata().await?.len();
                }
            }
        }
        
        Ok(ObjectStoreStats {
            blob_count,
            commit_count,
            total_size,
        })
    }
    
    fn get_blob_path(&self, hash: &Hash) -> PathBuf {
        let hash_str = hash.as_str();
        let (prefix, suffix) = hash_str.split_at(2);
        self.objects_path.join("blobs").join(prefix).join(format!("{}.json", suffix))
    }
    
    fn get_commit_path(&self, commit_id: &CommitId) -> PathBuf {
        self.objects_path.join("commits").join(format!("{}.json", commit_id))
    }
}

#[derive(Debug)]
pub struct ObjectStoreStats {
    pub blob_count: u64,
    pub commit_count: u64,
    pub total_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::core::intention::Intention;
    
    #[tokio::test]
    async fn test_blob_storage() {
        let temp_dir = TempDir::new().unwrap();
        let store = ObjectStore::new(&temp_dir.path().join("objects"));
        
        let content = b"Hello, world!";
        let hash = Hash::new(content);
        
        // Store blob
        store.store_blob(&hash, content.to_vec()).await.unwrap();
        assert!(store.blob_exists(&hash).await);
        
        // Load blob
        let loaded_blob = store.load_blob(&hash).await.unwrap();
        assert_eq!(loaded_blob.content, content);
        assert_eq!(loaded_blob.id, hash);
    }
    
    #[tokio::test]
    async fn test_commit_storage() {
        let temp_dir = TempDir::new().unwrap();
        let store = ObjectStore::new(&temp_dir.path().join("objects"));
        
        let intention = Intention::new(
            "Test commit".to_string(),
            None,
            None,
            0.8
        );
        
        let commit = FlowCommit::new(
            intention,
            vec![],
            Hash::new(b"tree"),
            vec![]
        );
        
        // Store commit
        store.store_commit(&commit).await.unwrap();
        assert!(store.commit_exists(&commit.id).await);
        
        // Load commit
        let loaded_commit = store.load_commit(&commit.id).await.unwrap();
        assert_eq!(loaded_commit.id.to_string(), commit.id.to_string());
        assert_eq!(loaded_commit.intention.goal, commit.intention.goal);
    }
    
    #[tokio::test]
    async fn test_object_store_stats() {
        let temp_dir = TempDir::new().unwrap();
        let store = ObjectStore::new(&temp_dir.path().join("objects"));
        
        // Initially empty
        let stats = store.get_stats().await.unwrap();
        assert_eq!(stats.blob_count, 0);
        assert_eq!(stats.commit_count, 0);
        
        // Add some objects
        let hash = Hash::new(b"content");
        store.store_blob(&hash, b"content".to_vec()).await.unwrap();
        
        let intention = Intention::new("Test".to_string(), None, None, 0.8);
        let commit = FlowCommit::new(intention, vec![], Hash::new(b"tree"), vec![]);
        store.store_commit(&commit).await.unwrap();
        
        let stats = store.get_stats().await.unwrap();
        assert_eq!(stats.blob_count, 1);
        assert_eq!(stats.commit_count, 1);
        assert!(stats.total_size > 0);
    }
}