//! Level 2 Type Checker for Synapse ASG
//!
//! Extension: Polymorphism (System F), algebraic datatypes, effect tracking.
//! This stubs the core entry point and adds extension points for new ASG v2 nodes.

mod types;
mod inference;
mod errors;

pub use types::*;
pub use errors::{TypeError, Result as TypeCheckResult};

use asg_core::AsgGraph;

/// Entry for extended type checking: supports TypeAbs, TypeApp, DataDef, DataMatch, EffectMeta.
pub fn check_and_annotate_graph_v2(graph: &mut AsgGraph) -> TypeCheckResult<()> {
    // For each node, match on v2 extensions.
    for (node_id, node) in &graph.nodes {
        match node.type_ {
            // ...existing node types...
            // Extension points:
            asg_core::NodeType::TypeAbs => {
                // TODO: System F style type abstraction (forall).
            }
            asg_core::NodeType::TypeApp => {
                // TODO: Instantiate type params, propagate types through body.
            }
            asg_core::NodeType::DataDef | asg_core::NodeType::DataCtor | asg_core::NodeType::DataMatch => {
                // TODO: ADT kinding, constructor type inference, exhaustiveness, match typing.
            }
            asg_core::NodeType::Unknown => {}
            _ => {
                // Existing inference logic for v1 nodes.
            }
        }
        // Effects extension:
        if let Some(effect_meta) = &node.effect_meta {
            // TODO: Record, analyze, or check effect tags in the type/effect context.
            let _effects : &Vec<String> = &effect_meta.effect_tags;
        }
    }
    Ok(())
}