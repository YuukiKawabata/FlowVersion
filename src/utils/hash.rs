use sha2::{Sha256, Digest};
use crate::core::objects::Hash;

pub fn hash_bytes(data: &[u8]) -> Hash {
    Hash::new(data)
}

pub fn hash_string(data: &str) -> Hash {
    Hash::new(data.as_bytes())
}

pub fn hash_file_content(content: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(b"blob ");
    hasher.update(content.len().to_string().as_bytes());
    hasher.update(b"\0");
    hasher.update(content);
    
    let result = hasher.finalize();
    Hash::from_string(hex::encode(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_consistency() {
        let data = b"test data";
        let hash1 = hash_bytes(data);
        let hash2 = hash_bytes(data);
        
        assert_eq!(hash1, hash2);
    }
    
    #[test]
    fn test_different_data_different_hash() {
        let hash1 = hash_string("data1");
        let hash2 = hash_string("data2");
        
        assert_ne!(hash1, hash2);
    }
    
    #[test]
    fn test_file_content_hash() {
        let content = b"file content";
        let hash = hash_file_content(content);
        
        // Should produce a valid hash
        assert!(!hash.as_str().is_empty());
        assert_eq!(hash.as_str().len(), 64); // SHA-256 hex length
    }
}