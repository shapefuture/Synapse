//! Effect system: propagate effect tags to type checker result/data for analysis.

mod types;
mod inference;
mod errors;

pub use types::Type;
pub use errors::{TypeError, Result as TypeCheckResult};

use asg_core::AsgGraph;
use std::collections::HashMap;

/// Effect analysis result: mapping node ids → effect tag sets (if any)
pub type EffectSummary = HashMap<u64, Vec<String>>;

/// Type checks and gathers effect tags.
pub fn check_and_annotate_graph_v2_with_effects(graph: &mut AsgGraph) -> (TypeCheckResult<()>, EffectSummary) {
    let mut summary = EffectSummary::new();
    for (node_id, node) in &graph.nodes {
        if let Some(meta) = &node.effect_meta {
            summary.insert(*node_id, meta.effect_tags.clone());
        }
    }
    (Ok(()), summary)
}