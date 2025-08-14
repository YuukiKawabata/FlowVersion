use crate::error::{FlowError, Result};
use crate::core::{
    intention::Intention,
    commit::FlowCommit,
    objects::{CommitId, Hash, FileChange},
};
use crate::storage::{
    object_store::ObjectStore,
    index::Index,
    config::Config,
};
use std::path::{Path, PathBuf};
use tracing::{info, debug, warn};

const FLOWVERSION_DIR: &str = ".flowversion";

pub struct Repository {
    root_path: PathBuf,
    object_store: ObjectStore,
    index: Index,
    config: Config,
}

impl Repository {
    pub async fn init(path: &Path, name: &str, ai_mode: &str) -> Result<()> {
        let flow_dir = path.join(FLOWVERSION_DIR);
        
        if flow_dir.exists() {
            return Err(FlowError::RepoAlreadyExists);
        }
        
        debug!("Creating FlowVersion directory structure");
        
        // Create directory structure
        std::fs::create_dir_all(&flow_dir)?;
        std::fs::create_dir_all(flow_dir.join("objects"))?;
        std::fs::create_dir_all(flow_dir.join("refs"))?;
        std::fs::create_dir_all(flow_dir.join("refs").join("streams"))?;
        
        // Create initial configuration
        let config = Config::new(name.to_string(), ai_mode.to_string());
        config.save(&flow_dir.join("config.json"))?;
        
        // Create empty index
        let index = Index::new();
        index.save(&flow_dir.join("index.json"))?;
        
        // Create HEAD file pointing to main stream
        std::fs::write(
            flow_dir.join("HEAD"),
            "ref: refs/streams/main\n"
        )?;
        
        info!("Repository initialized successfully");
        Ok(())
    }
    
    pub fn is_initialized(path: &Path) -> Result<bool> {
        let flow_dir = path.join(FLOWVERSION_DIR);
        Ok(flow_dir.exists() && flow_dir.is_dir())
    }
    
    pub async fn open(path: &Path) -> Result<Self> {
        let flow_dir = path.join(FLOWVERSION_DIR);
        
        if !Self::is_initialized(path)? {
            return Err(FlowError::RepoNotInitialized);
        }
        
        let config = Config::load(&flow_dir.join("config.json"))?;
        let index = Index::load(&flow_dir.join("index.json"))?;
        let object_store = ObjectStore::new(&flow_dir.join("objects"));
        
        Ok(Self {
            root_path: path.to_path_buf(),
            object_store,
            index,
            config,
        })
    }
    
    pub async fn add_file(&self, file_path: &str) -> Result<()> {
        let full_path = self.root_path.join(file_path);
        
        if !full_path.exists() {
            return Err(FlowError::FileNotFound(file_path.to_string()));
        }
        
        debug!("Adding file to index: {}", file_path);
        
        // Read file content
        let content = std::fs::read(&full_path)?;
        let hash = Hash::new(&content);
        
        // Store content as blob
        self.object_store.store_blob(&hash, content).await?;
        
        // Add to index
        self.index.add_file(file_path.to_string(), hash, full_path.metadata()?.len()).await?;
        
        Ok(())
    }
    
    pub async fn commit_with_intention(&self, intention: Intention) -> Result<CommitId> {
        debug!("Creating commit with intention: {}", intention.goal);
        
        // Validate intention
        intention.validate()?;
        
        // Get staged changes from index
        let staged_files = self.index.get_staged_files().await?;
        
        if staged_files.is_empty() {
            warn!("No files staged for commit");
            return Err(FlowError::InvalidRepoState("No files staged for commit".to_string()));
        }
        
        // Create file changes
        let mut changes = Vec::new();
        for (path, entry) in staged_files {
            let change = FileChange::new_added(path, entry.hash);
            changes.push(change);
        }
        
        // Create tree object
        let tree_hash = self.create_tree_from_changes(&changes).await?;
        
        // Get parent commits (current HEAD)
        let parent_commits = self.get_current_head_commits().await?;
        
        // Create commit
        let commit = FlowCommit::new(intention, changes, tree_hash, parent_commits);
        
        // Store commit
        self.object_store.store_commit(&commit).await?;
        
        // Update HEAD reference
        self.update_head(&commit.id).await?;
        
        // Clear index
        self.index.clear().await?;
        
        info!("Commit created: {}", commit.id);
        Ok(commit.id)
    }
    
    pub async fn get_log(&self) -> Result<Vec<FlowCommit>> {
        debug!("Retrieving commit log");
        
        let head_commit_id = self.get_head_commit_id().await?;
        
        if head_commit_id.is_none() {
            return Ok(Vec::new());
        }
        
        let mut commits = Vec::new();
        let mut current_id = head_commit_id;
        
        while let Some(commit_id) = current_id {
            let commit = self.object_store.load_commit(&commit_id).await?;
            
            // Get next commit (first parent)
            current_id = commit.parent_commits.first().cloned();
            
            commits.push(commit);
        }
        
        Ok(commits)
    }
    
    async fn create_tree_from_changes(&self, changes: &[FileChange]) -> Result<Hash> {
        // For now, create a simple hash from all file hashes
        let mut data = Vec::new();
        for change in changes {
            data.extend_from_slice(change.content_hash.as_str().as_bytes());
            data.extend_from_slice(change.path.as_bytes());
        }
        Ok(Hash::new(&data))
    }
    
    async fn get_current_head_commits(&self) -> Result<Vec<CommitId>> {
        match self.get_head_commit_id().await? {
            Some(id) => Ok(vec![id]),
            None => Ok(Vec::new()),
        }
    }
    
    async fn get_head_commit_id(&self) -> Result<Option<CommitId>> {
        let head_path = self.root_path.join(FLOWVERSION_DIR).join("HEAD");
        
        if !head_path.exists() {
            return Ok(None);
        }
        
        let head_content = std::fs::read_to_string(&head_path)?;
        
        if head_content.starts_with("ref: ") {
            // Symbolic reference
            let ref_path = head_content.trim_start_matches("ref: ").trim();
            let ref_file = self.root_path.join(FLOWVERSION_DIR).join(ref_path);
            
            if ref_file.exists() {
                let commit_id_str = std::fs::read_to_string(&ref_file)?;
                let uuid = uuid::Uuid::parse_str(commit_id_str.trim())
                    .map_err(|_| FlowError::InvalidCommitId(commit_id_str))?;
                Ok(Some(CommitId::from_uuid(uuid)))
            } else {
                Ok(None)
            }
        } else {
            // Direct commit ID
            let uuid = uuid::Uuid::parse_str(head_content.trim())
                .map_err(|_| FlowError::InvalidCommitId(head_content))?;
            Ok(Some(CommitId::from_uuid(uuid)))
        }
    }
    
    async fn update_head(&self, commit_id: &CommitId) -> Result<()> {
        let head_path = self.root_path.join(FLOWVERSION_DIR).join("HEAD");
        let head_content = std::fs::read_to_string(&head_path)?;
        
        if head_content.starts_with("ref: ") {
            // Update the reference
            let ref_path = head_content.trim_start_matches("ref: ").trim();
            let ref_file = self.root_path.join(FLOWVERSION_DIR).join(ref_path);
            
            // Create directory if it doesn't exist
            if let Some(parent) = ref_file.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            std::fs::write(&ref_file, commit_id.to_string())?;
        } else {
            // Update HEAD directly
            std::fs::write(&head_path, commit_id.to_string())?;
        }
        
        Ok(())
    }
    
    pub fn get_config(&self) -> &Config {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_repository_init() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        
        assert!(!Repository::is_initialized(path).unwrap());
        
        Repository::init(path, "test-repo", "local").await.unwrap();
        
        assert!(Repository::is_initialized(path).unwrap());
        
        let flow_dir = path.join(".flowversion");
        assert!(flow_dir.join("objects").exists());
        assert!(flow_dir.join("refs").exists());
        assert!(flow_dir.join("config.json").exists());
        assert!(flow_dir.join("index.json").exists());
        assert!(flow_dir.join("HEAD").exists());
    }
    
    #[tokio::test]
    async fn test_repository_open() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        
        // Should fail before init
        assert!(Repository::open(path).await.is_err());
        
        // Initialize and open
        Repository::init(path, "test-repo", "local").await.unwrap();
        let repo = Repository::open(path).await.unwrap();
        
        assert_eq!(repo.config.name, "test-repo");
        assert_eq!(repo.config.ai_mode, "local");
    }
}