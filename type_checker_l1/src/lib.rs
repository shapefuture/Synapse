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
/// Returns `Ok(())` if type checking succeeds, otherwise a vector of type errors.
pub fn check_and_annotate_graph(graph: &mut AsgGraph) -> TypeCheckResult<()> {
    // This is a stub; real implementation forthcoming
    Ok(())
}