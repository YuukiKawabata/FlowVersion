use thiserror::Error;

#[derive(Debug, Error)]
pub enum FlowError {
    // Repository errors
    #[error("Repository not initialized")]
    RepoNotInitialized,
    
    #[error("Repository already exists")]
    RepoAlreadyExists,
    
    #[error("Invalid repository state: {0}")]
    InvalidRepoState(String),
    
    #[error("Working directory is not clean")]
    WorkingDirectoryNotClean,
    
    // Object storage errors
    #[error("Object not found: {0}")]
    ObjectNotFound(String),
    
    #[error("Object already exists: {0}")]
    ObjectAlreadyExists(String),
    
    #[error("Invalid object hash: {0}")]
    InvalidObjectHash(String),
    
    // File system errors
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    // Serialization errors
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    // AI service errors
    #[error("AI service unavailable")]
    AIServiceUnavailable,
    
    #[error("AI analysis failed: {0}")]
    AIAnalysisFailed(String),
    
    #[error("Invalid AI response: {0}")]
    InvalidAIResponse(String),
    
    // Network errors
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    // Merge conflict errors
    #[error("Merge conflict in files: {0:?}")]
    MergeConflict(Vec<String>),
    
    #[error("Cannot resolve conflict automatically")]
    ConflictResolutionFailed,
    
    // Stream (branch) errors
    #[error("Stream not found: {0}")]
    StreamNotFound(String),
    
    #[error("Stream already exists: {0}")]
    StreamAlreadyExists(String),
    
    #[error("Invalid stream name: {0}")]
    InvalidStreamName(String),
    
    // Quantum branch errors
    #[error("Quantum state already collapsed")]
    QuantumStateCollapsed,
    
    #[error("Invalid quantum feature: {0}")]
    InvalidQuantumFeature(String),
    
    // Validation errors
    #[error("Invalid intention: {0}")]
    InvalidIntention(String),
    
    #[error("Invalid confidence score: {0} (must be between 0.0 and 1.0)")]
    InvalidConfidenceScore(f32),
    
    #[error("Invalid commit ID: {0}")]
    InvalidCommitId(String),
    
    // Configuration errors
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Missing configuration: {0}")]
    MissingConfig(String),
    
    // Generic errors
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Operation cancelled by user")]
    OperationCancelled,
    
    #[error("Feature not implemented: {0}")]
    NotImplemented(String),
}

impl FlowError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            FlowError::AIServiceUnavailable => true,
            FlowError::NetworkError(_) => true,
            FlowError::ConfigError(_) => true,
            _ => false,
        }
    }
    
    pub fn exit_code(&self) -> i32 {
        match self {
            FlowError::RepoNotInitialized => 128,
            FlowError::RepoAlreadyExists => 128,
            FlowError::FileNotFound(_) => 1,
            FlowError::PermissionDenied(_) => 13,
            FlowError::MergeConflict(_) => 1,
            FlowError::OperationCancelled => 130,
            _ => 1,
        }
    }
}

pub type Result<T> = std::result::Result<T, FlowError>;