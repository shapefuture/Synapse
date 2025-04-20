//! Type effect checking: checks effect tag flows and reports violations.

mod types;
mod inference;
mod errors;

pub use types::Type;
pub use errors::{TypeError, Result as TypeCheckResult};

use asg_core::AsgGraph;
use std::collections::HashSet;

/// Type checks and checks flow of disallowed effects. Fails if effect tags are violated.
pub fn check_and_annotate_graph_v2_with_effects_check(graph: &mut AsgGraph, allowed_effects: &[&str]) -> TypeCheckResult<()> {
    let allowed: HashSet<String> = allowed_effects.iter().map(|s| s.to_string()).collect();
    for (node_id, node) in &graph.nodes {
        if let Some(meta) = &node.effect_meta {
            for eff in &meta.effect_tags {
                if !allowed.contains(eff) {
                    return Err(errors::TypeError::Unimplemented); // Or introduce a better error type
                }
            }
        }
    }
    Ok(())
}