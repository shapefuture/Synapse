//! Level 1 Type Checker for Synapse ASG
//!
//! Implements Hindley-Milner style type inference and annotates
//! ASG nodes with inferred type information.

mod types;
mod unification;
mod inference;
mod errors;

pub use types::{Type, TypeScheme};
pub use errors::{TypeError, Result as TypeCheckResult};

use asg_core::AsgGraph;

/// The main entrypoint for type checking: checks and annotates the graph.
///
/// Returns `Ok(())` if type checking succeeds, otherwise an error if type checking fails.
pub fn check_and_annotate_graph(graph: &mut AsgGraph) -> TypeCheckResult<()> {
    // For now, just infer type for the root node and return error if it fails.
    let mut ctx = inference::TypingContext::new();
    let mut subst = std::collections::HashMap::new();
    let mut fresh_counter = 0u64;
    let root = graph.root_node_id();
    let ty = inference::infer(&mut ctx, root, graph, &mut subst, &mut fresh_counter)?;

    // TODO: Actually annotate the ASG nodes with inferred type info, e.g.
    // for (node_id, node) in &mut graph.nodes {
    //     node.inferred_type_id = ... // would link to a TypeNode
    // }
    // This will be handled post-schema update.

    Ok(())
}