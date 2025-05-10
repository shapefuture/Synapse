//! Level 1 Type Checker for Synapse ASG
//!
//! Implements Hindley-Milner style type inference and annotates
//! ASG nodes with inferred type information. Exposes API for checking and retrieval
//! of node types.

mod types;
mod unification;
mod inference;
mod errors;

pub use types::{Type, TypeScheme};
pub use errors::{TypeError, Result as TypeCheckResult};

use asg_core::AsgGraph;
use std::collections::HashMap;

/// Result of a full type check: mapping node IDs to their inferred types.
pub type TypeCheckMap = HashMap<u64, Type>;

/// Main entrypoint for type checking: checks and annotates the graph.
/// 
/// Populates and returns a map: node_id → inferred type.
/// Returns an error if type checking fails for any node.
///
/// When the ASG schema allows, this can also annotate nodes with inferred type IDs.
pub fn check_and_annotate_graph(graph: &mut AsgGraph) -> TypeCheckResult<TypeCheckMap> {
    let mut ctx = inference::TypingContext::new();
    let mut subst = HashMap::new();
    let mut fresh_counter = 0u64;

    let mut inferred_types: TypeCheckMap = HashMap::new();

    // DFS traversal of all nodes reachable from root.
    let root_id = graph.root_node_id();
    let mut stack = vec![root_id];
    let mut visited = HashMap::new();

    while let Some(nid) = stack.pop() {
        if visited.contains_key(&nid) {
            continue;
        }
        visited.insert(nid, true);
        // Infer the type for this node
        match inference::infer(&mut ctx, nid, graph, &mut subst, &mut fresh_counter) {
            Ok(ty) => {
                inferred_types.insert(nid, ty);
            }
            Err(err) => {
                // Return error immediately; if you want to collect all errors, replace with Vec<TypeError>
                return Err(err);
            }
        }
        // Add child nodes for visitation, if any
        if let Some(node) = graph.nodes.get(&nid) {
            for child in node.all_child_node_ids() {
                stack.push(child);
            }
        }
    }

    // TYPE ANNOTATION STUB (Schema support pending):
    /*
    for (node_id, ty) in &inferred_types {
        if let Some(node) = graph.nodes.get_mut(node_id) {
            // node.inferred_type_id = ...; // If supported by schema
        }
    }
    */

    Ok(inferred_types)
}

/// Convenience helper: get the inferred type for a node after type checking.
/// If called before type check, runs the pass (may be slow!).
pub fn get_inferred_type_for_node(node_id: u64, graph: &AsgGraph) -> Option<Type> {
    match check_and_annotate_graph(&mut graph.clone()) {
        Ok(types) => types.get(&node_id).cloned(),
        Err(_) => None,
    }
}

// ---- Helper Trait (for traversal) ----
use asg_core::AsgNode;

trait NodeChildren {
    /// Returns all direct child node IDs for traversal.
    fn all_child_node_ids(&self) -> Vec<u64>;
}

impl NodeChildren for AsgNode {
    fn all_child_node_ids(&self) -> Vec<u64> {
        let mut ids = Vec::new();
        use asg_core::NodeType::*;
        match self.type_ {
            NodeType::TermVariable => {
                if let Some(v) = &self.term_variable {
                    ids.push(v.definition_node_id);
                }
            }
            NodeType::TermLambda => {
                if let Some(lambda) = &self.term_lambda {
                    ids.push(lambda.binder_variable_node_id);
                    ids.push(lambda.body_node_id);
                    if let Some(tid) = lambda.type_annotation_id {
                        ids.push(tid);
                    }
                }
            }
            NodeType::TermApplication => {
                if let Some(app) = &self.term_application {
                    ids.push(app.function_node_id);
                    ids.push(app.argument_node_id);
                }
            }
            NodeType::TermRef => {
                if let Some(r) = &self.term_ref {
                    ids.push(r.init_value_node_id);
                }
            }
            NodeType::TermDeref => {
                if let Some(r) = &self.term_deref {
                    ids.push(r.ref_node_id);
                }
            }
            NodeType::TermAssign => {
                if let Some(r) = &self.term_assign {
                    ids.push(r.ref_node_id);
                    ids.push(r.value_node_id);
                }
            }
            NodeType::PrimitiveOp => {
                if let Some(op) = &self.primitive_op {
                    ids.extend_from_slice(&op.argument_node_ids);
                }
            }
            _ => {}
        }
        ids
    }
}