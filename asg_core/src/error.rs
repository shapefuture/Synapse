//! Error handling for the ASG Core library.

use thiserror::Error;

/// Result type for ASG Core operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for ASG Core operations.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Node not found: {0}")]
    NodeNotFound(u64),
    
    #[error("Invalid node reference: {0}")]
    InvalidNodeReference(u64),
    
    #[error("Node type mismatch: expected {expected}, found {found}")]
    NodeTypeMismatch {
        expected: String,
        found: String,
    },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    
    #[error("Protobuf error: {0}")]
    Protobuf(#[from] prost::EncodeError),
    
    #[error("Protobuf decode error: {0}")]
    ProtobufDecode(#[from] prost::DecodeError),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Graph integrity error: {0}")]
    GraphIntegrity(String),
}