//! Type checking errors.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, TypeError>;

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("Unification failed between {0:?} and {1:?}")]
    UnificationFail(super::types::Type, super::types::Type),

    #[error("Occurs check failed")]
    OccursCheck,

    #[error("Undefined variable (node id: {0})")]
    UndefinedVariable(u64),

    #[error("Application type mismatch at node {0}")]
    ApplicationMismatch(u64),

    #[error("Unimplemented type checking logic")]
    Unimplemented,
}