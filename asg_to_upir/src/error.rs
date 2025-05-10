//! ASG-to-UPIR lowering error types.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, LoweringError>;

#[derive(Debug, Error)]
pub enum LoweringError {
    #[error("Node {0} not recognized for lowering")]
    NodeNotLowerable(u64),
    #[error("Type for node {0} not found")]
    MissingType(u64),
    #[error("Unimplemented lowering for {0}")]
    Unimplemented(String),
}