//! Level 1 Type Checker for Synapse ASG
//!
//! Implements Hindley-Milner style type inference and annotates
//! ASG nodes with inferred type information. Exposes API for checking and (when schema allows) annotation of type info.

mod types;
mod unification;
mod inference;
mod errors;

pub use types::{Type, TypeScheme};
pub use errors::{TypeError, Result as TypeCheckResult};

use asg_core::AsgGraph;
use std::collections::HashMap;

/// Main entrypoint for type checking: checks and annotates the graph.
///
/// Returns `Ok(())` if type checking succeeds, otherwise an error if type checking fails.
/// 
/// When the ASG schema allows, this will annotate nodes with inferred type information.
pub fn check_and_annotate_graph(graph: &mut AsgGraph) -> TypeCheckResult<()> {
    // Run HM type inference from the root node.
    let mut ctx = inference::TypingContext::new();
    let mut subst = HashMap::new();
    let mut fresh_counter = 0u64;
    let root = graph.root_node_id();
    let ty = inference::infer(&mut ctx, root, graph, &mut subst, &mut fresh_counter)?;

    // === TYPE ANNOTATION LOGIC (pending ASG schema support) ===
    // For each node, you would attach inferred type data, e.g.:
    /*
    for (node_id, node) in &mut graph.nodes {
        // 1. Look up the final inferred type for this node (if stored)
        // 2. Add or set node.inferred_type_id (linking to a TypeNode)
    }
    */
    // TODO: Once schema supports this, enable actual annotation.
    let _ = ty;
    Ok(())
}

/// Convenience helper to get the inferred type for a particular node, given a checked ASG.
/// (This is a stub until full node-to-type mapping is tracked).
pub fn get_inferred_type_for_node(_node_id: u64, _graph: &AsgGraph) -> Option<Type> {
    // TODO: With schema/type annotation support, implement storage/lookup.
    None
}