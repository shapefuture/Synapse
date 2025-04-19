//! Core library for the Abstract Semantic Graph (ASG) used in the Synapse compiler.
//!
//! This library provides the foundational data structures and operations for building,
//! manipulating, serializing, deserializing, and hashing ASG instances.

mod error;
mod graph;
mod hash;
mod serde;

// Include the generated protobuf code
pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/synapse.asg.v1.rs"));
}

// Re-export key components
pub use error::{Error, Result};
pub use graph::AsgGraph;
pub use hash::{hash_node, HashDigest};
pub use serde::{load_asg_binary, save_asg_binary, load_asg_json, save_asg_json};

// Public API types from generated code
pub use generated::{
    AsgNode, NodeType, TermVariable, TermLambda, TermApplication,
    LiteralInt, LiteralBool, PrimitiveOp, TermRef, TermDeref, TermAssign,
    EffectPerform, ProofObligation, TypeNode, TypeKind, Metadata, SourceLocation,
};