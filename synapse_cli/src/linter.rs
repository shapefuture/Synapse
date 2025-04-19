//! Linter for the Synapse compiler.
//!
//! This module provides Level 0 (structural) linting for Synapse ASGs.

use std::collections::{HashMap, HashSet};

use asg_core::{AsgGraph, NodeType};

/// A lint error found in an ASG.
#[derive(Debug, Clone)]
pub struct LintError {
    /// Error code (e.g., L001)
    pub code: String,
    /// Error message
    pub message: String,
    /// Node ID where the error was found
    pub node_id: u64,
    /// Source location if available
    pub source_location: Option<asg_core::SourceLocation>,
}

impl std::fmt::Display for LintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(loc) = &self.source_location {
            write!(
                f,
                "[{}] {}: {}:{}:{}",
                self.code, self.message, loc.filename, loc.start_line, loc.start_col
            )
        } else {
            write!(f, "[{}] {} at node {}", self.code, self.message, self.node_id)
        }
    }
}

/// Lints an ASG for structural issues.
pub fn lint_graph(graph: &AsgGraph) -> Vec<LintError> {
    let mut errors = Vec::new();
    
    // Check graph structural integrity
    check_graph_integrity(graph, &mut errors);
    
    // Perform scope checking
    check_scopes(graph, &mut errors);
    
    // Check for simple type mismatches
    check_simple_types(graph, &mut errors);
    
    errors
}

/// Checks the structural integrity of the graph.
fn check_graph_integrity(graph: &AsgGraph, errors: &mut Vec<LintError>) {
    let nodes = graph.nodes();
    
    // Check that the root node exists if set
    if let Some(root_id) = graph.root_id() {
        if !nodes.contains_key(&root_id) {
            errors.push(LintError {
                code: "L001".to_string(),
                message: format!("Root node {} not found in graph", root_id),
                node_id: root_id,
                source_location: None,
            });
            return; // Early return if the root is invalid
        }
    }
    
    // Check each node for references to non-existent nodes
    for (&node_id, node) in nodes {
        match node.r#type {
            typ if typ == NodeType::TERM_VARIABLE as i32 => {
                if let Some(asg_core::generated::asg_node::Content::TermVariable(var)) = &node.content {
                    if var.definition_node_id != 0 && !nodes.contains_key(&var.definition_node_id) {
                        errors.push(LintError {
                            code: "L002".to_string(),
                            message: format!(
                                "Variable '{}' references non-existent definition node {}",
                                var.name, var.definition_node_id
                            ),
                            node_id,
                            source_location: get_source_location(graph, node_id),
                        });
                    }
                }
            }
            typ if typ == NodeType::TERM_LAMBDA as i32 => {
                if let Some(asg_core::generated::asg_node::Content::TermLambda(lambda)) = &node.content {
                    if !nodes.contains_key(&lambda.binder_variable_node_id) {
                        errors.push(LintError {
                            code: "L003".to_string(),
                            message: format!(
                                "Lambda references non-existent binder node {}",
                                lambda.binder_variable_node_id
                            ),
                            node_id,
                            source_location: get_source_location(graph, node_id),
                        });
                    }
                    if !nodes.contains_key(&lambda.body_node_id) {
                        errors.push(LintError {
                            code: "L004".to_string(),
                            message: format!(
                                "Lambda references non-existent body node {}",
                                lambda.body_node_id
                            ),
                            node_id,
                            source_location: get_source_location(graph, node_id),
                        });
                    }
                    if lambda.type_annotation_id != 0 && !nodes.contains_key(&lambda.type_annotation_id) {
                        errors.push(LintError {
                            code: "L005".to_string(),
                            message: format!(
                                "Lambda references non-existent type annotation node {}",
                                lambda.type_annotation_id
                            ),
                            node_id,
                            source_location: get_source_location(graph, node_id),
                        });
                    }
                }
            }
            typ if typ == NodeType::TERM_APPLICATION as i32 => {
                if let Some(asg_core::generated::asg_node::Content::TermApplication(app)) = &node.content {
                    if !nodes.contains_key(&app.function_node_id) {
                        errors.push(LintError {
                            code: "L006".to_string(),
                            message: format!(
                                "Application references non-existent function node {}",
                                app.function_node_id
                            ),
                            node_id,
                            source_location: get_source_location(graph, node_id),
                        });
                    }
                    if !nodes.contains_key(&app.argument_node_id) {
                        errors.push(LintError {
                            code: "L007".to_string(),
                            message: format!(
                                "Application references non-existent argument node {}",
                                app.argument_node_id
                            ),
                            node_id,
                            source_location: get_source_location(graph, node_id),
                        });
                    }
                }
            }
            // Similar checks for other node types...
            _ => {} // Skip node types that don't have direct references
        }
    }
}

/// Checks variable scoping.
fn check_scopes(graph: &AsgGraph, errors: &mut Vec<LintError>) {
    // Start from the root if it exists
    if let Some(root_id) = graph.root_id() {
        // Build a map of variables to their definitions
        let mut scope_info = HashMap::new();
        build_scope_info(graph, root_id, &mut scope_info, errors);
    }
}

/// Builds scope information for the ASG.
fn build_scope_info(
    graph: &AsgGraph, 
    node_id: u64, 
    scope_info: &mut HashMap<u64, u64>, // Maps variable node ID -> definition node ID
    errors: &mut Vec<LintError>,
) {
    let node = match graph.get_node(node_id) {
        Some(node) => node,
        None => return, // Node not found, should be caught by integrity check
    };
    
    match node.r#type {
        typ if typ == NodeType::TERM_VARIABLE as i32 => {
            if let Some(asg_core::generated::asg_node::Content::TermVariable(var)) = &node.content {
                // Check if the variable's definition is in scope
                if var.definition_node_id != 0 && !scope_info.contains_key(&var.definition_node_id) {
                    errors.push(LintError {
                        code: "L008".to_string(),
                        message: format!(
                            "Variable '{}' references definition node {} which is not in scope",
                            var.name, var.definition_node_id
                        ),
                        node_id,
                        source_location: get_source_location(graph, node_id),
                    });
                }
            }
        }
        typ if typ == NodeType::TERM_LAMBDA as i32 => {
            if let Some(asg_core::generated::asg_node::Content::TermLambda(lambda)) = &node.content {
                // Add the lambda's parameter to the scope
                scope_info.insert(lambda.binder_variable_node_id, node_id);
                
                // Check the lambda's body with the updated scope
                build_scope_info(graph, lambda.body_node_id, scope_info, errors);
                
                // Remove the parameter from scope after checking the body
                scope_info.remove(&lambda.binder_variable_node_id);
            }
        }
        typ if typ == NodeType::TERM_APPLICATION as i32 => {
            if let Some(asg_core::generated::asg_node::Content::TermApplication(app)) = &node.content {
                // Check the function and argument
                build_scope_info(graph, app.function_node_id, scope_info, errors);
                build_scope_info(graph, app.argument_node_id, scope_info, errors);
            }
        }
        // Handle other node types recursively
        _ => {
            // For simplicity, we'll skip the exhaustive recursive traversal here
            // A complete implementation would handle all node types
        }
    }
}

/// Checks for simple type mismatches.
fn check_simple_types(graph: &AsgGraph, errors: &mut Vec<LintError>) {
    // Traverse the graph and check for obvious type errors
    let nodes = graph.nodes();
    
    for (&node_id, node) in nodes {
        match node.r#type {
            typ if typ == NodeType::TERM_APPLICATION as i32 => {
                if let Some(asg_core::generated::asg_node::Content::TermApplication(app)) = &node.content {
                    // Check if the function is a non-lambda value (like a literal)
                    if let Some(func_node) = graph.get_node(app.function_node_id) {
                        if func_node.r#type == NodeType::LITERAL_INT as i32 ||
                           func_node.r#type == NodeType::LITERAL_BOOL as i32 {
                            errors.push(LintError {
                                code: "L009".to_string(),
                                message: "Cannot apply arguments to a non-function value".to_string(),
                                node_id,
                                source_location: get_source_location(graph, node_id),
                            });
                        }
                    }
                }
            }
            typ if typ == NodeType::TERM_DEREF as i32 => {
                if let Some(asg_core::generated::asg_node::Content::TermDeref(deref)) = &node.content {
                    // Check if we're dereferencing a non-reference value
                    if let Some(ref_node) = graph.get_node(deref.ref_node_id) {
                        if ref_node.r#type == NodeType::LITERAL_INT as i32 ||
                           ref_node.r#type == NodeType::LITERAL_BOOL as i32 ||
                           ref_node.r#type == NodeType::TERM_LAMBDA as i32 {
                            errors.push(LintError {
                                code: "L010".to_string(),
                                message: "Cannot dereference a non-reference value".to_string(),
                                node_id,
                                source_location: get_source_location(graph, node_id),
                            });
                        }
                    }
                }
            }
            typ if typ == NodeType::TERM_ASSIGN as i32 => {
                if let Some(asg_core::generated::asg_node::Content::TermAssign(assign)) = &node.content {
                    // Check if we're assigning to a non-reference value
                    if let Some(ref_node) = graph.get_node(assign.ref_node_id) {
                        if ref_node.r#type != NodeType::TERM_DEREF as i32 &&
                           ref_node.r#type != NodeType::TERM_VARIABLE as i32 {
                            errors.push(LintError {
                                code: "L011".to_string(),
                                message: "Left-hand side of assignment must be a reference or variable".to_string(),
                                node_id,
                                source_location: get_source_location(graph, node_id),
                            });
                        }
                    }
                }
            }
            // Other type checks would go here
            _ => {}
        }
    }
}

/// Gets the source location for a node.
fn get_source_location(graph: &AsgGraph, node_id: u64) -> Option<asg_core::SourceLocation> {
    for (_, node) in graph.nodes() {
        if let Some(asg_core::generated::asg_node::Content::Metadata(metadata)) = &node.content {
            if metadata.node_id == node_id {
                return metadata.source_location.clone();
            }
        }
    }
    None
}