use serde::{Deserialize, Serialize};
use crate::core::objects::{StreamId, CommitId};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamMode {
    Linear,     // Traditional linear history
    Parallel,   // Allow parallel development
    Quantum,    // Quantum superposition of features
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stream {
    pub id: StreamId,
    pub name: String,
    pub mode: StreamMode,
    pub head_commit: Option<CommitId>,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
    pub metadata: serde_json::Value,
}

impl Stream {
    pub fn new(name: String, mode: StreamMode) -> Self {
        Self {
            id: StreamId::new(),
            name,
            mode,
            head_commit: None,
            created_at: Utc::now(),
            description: None,
            metadata: serde_json::Value::Null,
        }
    }
    
    pub fn main_stream() -> Self {
        Self::new("main".to_string(), StreamMode::Linear)
    }
    
    pub fn set_head(&mut self, commit_id: CommitId) {
        self.head_commit = Some(commit_id);
    }
    
    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }
    
    pub fn is_main(&self) -> bool {
        self.name == "main"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stream_creation() {
        let stream = Stream::new("feature/auth".to_string(), StreamMode::Linear);
        
        assert_eq!(stream.name, "feature/auth");
        assert!(matches!(stream.mode, StreamMode::Linear));
        assert!(stream.head_commit.is_none());
        assert!(!stream.is_main());
    }
    
    #[test]
    fn test_main_stream() {
        let stream = Stream::main_stream();
        
        assert_eq!(stream.name, "main");
        assert!(stream.is_main());
    }
    
    #[test]
    fn test_stream_operations() {
        let mut stream = Stream::new("test".to_string(), StreamMode::Linear);
        let commit_id = CommitId::new();
        
        stream.set_head(commit_id.clone());
        stream.set_description("Test stream".to_string());
        
        assert_eq!(stream.head_commit, Some(commit_id));
        assert_eq!(stream.description, Some("Test stream".to_string()));
    }
}