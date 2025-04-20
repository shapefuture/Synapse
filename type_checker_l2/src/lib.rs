//! Level 2 Type Checker for Synapse ASG—System F, ADTs (polymorphism extension)

mod types;
mod inference;
mod errors;

pub use types::Type;
pub use errors::{TypeError, Result as TypeCheckResult};

use asg_core::AsgGraph;

/// Extended type check for polymorphism; handles TypeAbs and TypeApp.
pub fn check_and_annotate_graph_v2(graph: &mut AsgGraph) -> TypeCheckResult<()> {
    for (node_id, node) in &graph.nodes {
        match node.type_ {
            asg_core::NodeType::TypeAbs | asg_core::NodeType::TypeApp => {
                let mut ctx = inference::TypeContext::new();
                let _ty = inference::infer(&mut ctx, *node_id, graph)?;
                // TODO: annotate node.type_scheme, etc.
            }
            _ => {} // For brevity, only handle polymorphic extension nodes here.
        }
    }
    Ok(())
}