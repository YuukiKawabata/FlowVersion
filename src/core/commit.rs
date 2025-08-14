use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::core::{intention::Intention, objects::{CommitId, Hash, FileChange}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowCommit {
    pub id: CommitId,
    pub intention: Intention,
    pub changes: Vec<FileChange>,
    pub tree_hash: Hash,
    pub parent_commits: Vec<CommitId>,
    pub timestamp: DateTime<Utc>,
    pub signature: Option<CommitSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSignature {
    pub algorithm: String,
    pub signature: String,
    pub public_key: String,
}

impl FlowCommit {
    pub fn new(
        intention: Intention,
        changes: Vec<FileChange>,
        tree_hash: Hash,
        parent_commits: Vec<CommitId>,
    ) -> Self {
        Self {
            id: CommitId::new(),
            intention,
            changes,
            tree_hash,
            parent_commits,
            timestamp: Utc::now(),
            signature: None,
        }
    }
    
    pub fn is_merge_commit(&self) -> bool {
        self.parent_commits.len() > 1
    }
    
    pub fn is_root_commit(&self) -> bool {
        self.parent_commits.is_empty()
    }
    
    pub fn short_id(&self) -> String {
        self.id.to_string()[..8].to_string()
    }
    
    pub fn get_changed_files(&self) -> Vec<String> {
        self.changes.iter()
            .map(|change| change.path.clone())
            .collect()
    }
    
    pub fn files_count(&self) -> usize {
        self.changes.len()
    }
    
    pub fn has_file(&self, path: &str) -> bool {
        self.changes.iter().any(|change| change.path == path)
    }
    
    pub fn sign(&mut self, _private_key: &str) -> crate::error::Result<()> {
        // TODO: Implement actual cryptographic signing
        // For now, this is a placeholder
        self.signature = Some(CommitSignature {
            algorithm: "EdDSA".to_string(),
            signature: "placeholder_signature".to_string(),
            public_key: "placeholder_public_key".to_string(),
        });
        Ok(())
    }
    
    pub fn verify_signature(&self) -> bool {
        // TODO: Implement actual signature verification
        // For now, just check if signature exists
        self.signature.is_some()
    }
    
    pub fn to_bytes(&self) -> crate::error::Result<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| crate::error::FlowError::SerializationError(e))
    }
    
    pub fn from_bytes(bytes: &[u8]) -> crate::error::Result<Self> {
        serde_json::from_slice(bytes)
            .map_err(|e| crate::error::FlowError::SerializationError(e))
    }
    
    pub fn calculate_commit_hash(&self) -> Hash {
        // Create a deterministic representation for hashing
        let mut data = Vec::new();
        data.extend_from_slice(self.id.to_string().as_bytes());
        data.extend_from_slice(self.intention.id.to_string().as_bytes());
        data.extend_from_slice(self.tree_hash.as_str().as_bytes());
        
        for parent in &self.parent_commits {
            data.extend_from_slice(parent.to_string().as_bytes());
        }
        
        data.extend_from_slice(self.timestamp.to_rfc3339().as_bytes());
        
        Hash::new(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::objects::{ChangeType};
    
    #[test]
    fn test_commit_creation() {
        let intention = Intention::new(
            "Test commit".to_string(),
            None,
            None,
            0.8
        );
        
        let change = FileChange {
            path: "test.txt".to_string(),
            change_type: ChangeType::Added,
            content_hash: Hash::new(b"test content"),
            previous_hash: None,
            mode: 0o644,
        };
        
        let tree_hash = Hash::new(b"tree content");
        let commit = FlowCommit::new(
            intention.clone(),
            vec![change],
            tree_hash.clone(),
            vec![]
        );
        
        assert_eq!(commit.intention.goal, intention.goal);
        assert_eq!(commit.tree_hash, tree_hash);
        assert!(commit.is_root_commit());
        assert!(!commit.is_merge_commit());
        assert_eq!(commit.files_count(), 1);
    }
    
    #[test]
    fn test_commit_serialization() {
        let intention = Intention::new(
            "Test serialization".to_string(),
            None,
            None,
            0.8
        );
        
        let commit = FlowCommit::new(
            intention,
            vec![],
            Hash::new(b"empty tree"),
            vec![]
        );
        
        let bytes = commit.to_bytes().unwrap();
        let deserialized = FlowCommit::from_bytes(&bytes).unwrap();
        
        assert_eq!(commit.id.to_string(), deserialized.id.to_string());
        assert_eq!(commit.intention.goal, deserialized.intention.goal);
    }
    
    #[test]
    fn test_merge_commit() {
        let intention = Intention::new(
            "Merge branches".to_string(),
            None,
            None,
            0.9
        );
        
        let parent1 = CommitId::new();
        let parent2 = CommitId::new();
        
        let commit = FlowCommit::new(
            intention,
            vec![],
            Hash::new(b"merge tree"),
            vec![parent1, parent2]
        );
        
        assert!(commit.is_merge_commit());
        assert!(!commit.is_root_commit());
    }
    
    #[test]
    fn test_commit_hash() {
        let intention = Intention::new(
            "Hash test".to_string(),
            None,
            None,
            0.8
        );
        
        let commit1 = FlowCommit::new(
            intention.clone(),
            vec![],
            Hash::new(b"tree1"),
            vec![]
        );
        
        let commit2 = FlowCommit::new(
            intention,
            vec![],
            Hash::new(b"tree2"),
            vec![]
        );
        
        let hash1 = commit1.calculate_commit_hash();
        let hash2 = commit2.calculate_commit_hash();
        
        // Different tree hashes should produce different commit hashes
        assert_ne!(hash1, hash2);
    }
}