//! Algorithm W type inference for Synapse ASG.

use super::types::{Type, TypeScheme};
use super::unification::SubstitutionMap;
use super::errors::{TypeError, Result};

use asg_core::AsgGraph;

use std::collections::HashMap;

pub struct TypingContext {
    // Maps ASG var node ID to type scheme
    pub bindings: HashMap<u64, TypeScheme>,
}

// Stub type inference
pub fn infer(
    _context: &TypingContext,
    _node_id: u64,
    _graph: &AsgGraph,
    _subst: &mut SubstitutionMap,
) -> Result<Type> {
    // TODO: Implement Hindley-Milner inference (Algorithm W)
    Err(TypeError::Unimplemented)
}