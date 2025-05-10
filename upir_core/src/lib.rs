//! UPIR Core Library for Synapse
//!
//! Defines main IR data structures, types, operations, and API.

pub mod ir;
pub mod types;
pub mod attributes;
pub mod dialects;

pub use crate::ir::*;
pub use crate::types::*;
pub use crate::attributes::*;