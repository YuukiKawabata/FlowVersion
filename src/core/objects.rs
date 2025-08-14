use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Hash(String);

impl Hash {
    pub fn new(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        Hash(hex::encode(result))
    }
    
    pub fn from_string(s: String) -> Self {
        Hash(s)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Hash {
    fn from(s: String) -> Self {
        Hash(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitId(Uuid);

impl Default for CommitId {
    fn default() -> Self {
        Self::new()
    }
}

impl CommitId {
    pub fn new() -> Self {
        CommitId(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        CommitId(uuid)
    }
    
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl std::fmt::Display for CommitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamId(Uuid);

impl Default for StreamId {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamId {
    pub fn new() -> Self {
        StreamId(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        StreamId(uuid)
    }
    
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl std::fmt::Display for StreamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed { from: String },
    Copied { from: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub change_type: ChangeType,
    pub content_hash: Hash,
    pub previous_hash: Option<Hash>,
    pub mode: u32, // File permissions
}

impl FileChange {
    pub fn new_added(path: String, content_hash: Hash) -> Self {
        Self {
            path,
            change_type: ChangeType::Added,
            content_hash,
            previous_hash: None,
            mode: 0o644, // Default file permissions
        }
    }
    
    pub fn new_modified(path: String, content_hash: Hash, previous_hash: Hash) -> Self {
        Self {
            path,
            change_type: ChangeType::Modified,
            content_hash,
            previous_hash: Some(previous_hash),
            mode: 0o644,
        }
    }
    
    pub fn new_deleted(path: String, previous_hash: Hash) -> Self {
        Self {
            path,
            change_type: ChangeType::Deleted,
            content_hash: Hash::from_string("0".repeat(64)), // Empty hash for deleted files
            previous_hash: Some(previous_hash),
            mode: 0o644,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeObject {
    pub id: Hash,
    pub entries: HashMap<String, TreeEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TreeEntry {
    Blob {
        hash: Hash,
        mode: u32,
    },
    Tree {
        hash: Hash,
    },
}

impl Default for TreeObject {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeObject {
    pub fn new() -> Self {
        Self {
            id: Hash::from_string("".to_string()), // Will be calculated when stored
            entries: HashMap::new(),
        }
    }
    
    pub fn add_blob(&mut self, path: String, hash: Hash, mode: u32) {
        self.entries.insert(path, TreeEntry::Blob { hash, mode });
    }
    
    pub fn add_tree(&mut self, path: String, hash: Hash) {
        self.entries.insert(path, TreeEntry::Tree { hash });
    }
    
    pub fn calculate_hash(&mut self) {
        let serialized = serde_json::to_vec(self).unwrap();
        self.id = Hash::new(&serialized);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobObject {
    pub id: Hash,
    pub content: Vec<u8>,
}

impl BlobObject {
    pub fn new(content: Vec<u8>) -> Self {
        let id = Hash::new(&content);
        Self { id, content }
    }
    
    pub fn from_string(content: String) -> Self {
        Self::new(content.into_bytes())
    }
    
    pub fn as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.content.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_creation() {
        let data = b"hello world";
        let hash1 = Hash::new(data);
        let hash2 = Hash::new(data);
        assert_eq!(hash1, hash2);
        
        let different_data = b"hello world!";
        let hash3 = Hash::new(different_data);
        assert_ne!(hash1, hash3);
    }
    
    #[test]
    fn test_file_change_creation() {
        let hash = Hash::new(b"content");
        let change = FileChange::new_added("test.txt".to_string(), hash.clone());
        
        match change.change_type {
            ChangeType::Added => {}
            _ => panic!("Expected Added change type"),
        }
        
        assert_eq!(change.path, "test.txt");
        assert_eq!(change.content_hash, hash);
        assert_eq!(change.previous_hash, None);
    }
    
    #[test]
    fn test_blob_object() {
        let content = "Hello, world!".to_string();
        let blob = BlobObject::from_string(content.clone());
        
        assert_eq!(blob.as_string().unwrap(), content);
        
        let blob2 = BlobObject::from_string(content);
        assert_eq!(blob.id, blob2.id);
    }
    
    #[test]
    fn test_tree_object() {
        let mut tree = TreeObject::new();
        let hash = Hash::new(b"test content");
        
        tree.add_blob("file.txt".to_string(), hash, 0o644);
        tree.calculate_hash();
        
        assert!(!tree.id.as_str().is_empty());
        assert!(tree.entries.contains_key("file.txt"));
    }
}