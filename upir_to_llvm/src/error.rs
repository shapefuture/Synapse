//! UPIR-to-LLVM lowering error types.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, LlvmLoweringError>;

#[derive(Debug, Error)]
pub enum LlvmLoweringError {
    #[error("Not implemented: {0}")]
    Unimplemented(String),
}