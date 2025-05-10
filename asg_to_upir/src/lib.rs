//! ASG → UPIR Lowering for Synapse:
//! Implements core compiler lowering pass from checked ASG to UPIR IR.

mod lowering;
mod error;

pub use lowering::*;
pub use error::*;