//! Level 0 linter for Synapse ASG.
//!
//! Performs basic structural validation of the ASG:
//! - Checks that referenced node IDs exist
//! - Validates variable binding and scope
//! - Detects simple type mismatches without full inference

use asg_core::{AsgGraph, AsgNode, NodeType};
use std::collections::{HashMap, HashSet};

/// Error codes for the linter.
pub enum LintErrorCode {
    /// Referenced node ID doesn't exist
    IntegrityError,
    /// Variable reference points to non-existent or invalid binder
    ScopeError,
    /// Application of a non-lambda value
    ApplicationError,
    /// Assignment to a non-reference
    AssignmentError,
}

impl LintErrorCode {
    /// Convert the error code to a string identifier
    pub fn as_str(&self) -> &'static str {
        match self {
            LintErrorCode::IntegrityError => "L001",
            LintErrorCode::ScopeError => "L002",
            LintErrorCode::ApplicationError => "L003",
            LintErrorCode::AssignmentError => "L004",
        }
    }
}

/// Location of lint error in source code
#[derive(Debug, Clone)]
pub struct SourceLocation {
    /// Source filename
    pub filename: Option<String>,
    /// Start line number (1-based)
    pub start_line: u32,
    /// Start column number (1-based)
    pub start_col: u32,
    /// End line number (1-based)
    pub end_line: u32,
    /// End column number (1-based)
    pub end_col: u32,
}

/// Lint error detected in ASG
#[derive(Debug)]
pub struct LintError {
    /// Error code (e.g., "L001")
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// ID of the problematic ASG node
    pub node_id: u64,
    /// Source code location if available
    pub location: Option<SourceLocation>,
}

/// Run all Level 0 linting checks on the ASG
pub fn lint_graph(graph: &AsgGraph) -> Vec<LintError> {
    let mut errors = Vec::new();
    
    // Collect various linting passes
    check_node_references(graph, &mut errors);
    check_variable_scopes(graph, &mut errors);
    check_simple_type_errors(graph, &mut errors);
    
    errors
}

/// Check that all node IDs referenced in the graph actually exist
fn check_node_references(graph: &AsgGraph, errors: &mut Vec<LintError>) {
    // Set of all node IDs that exist in the graph
    let existing_nodes: HashSet<u64> = graph.nodes.keys().cloned().collect();
    
    // Check each node for references to other nodes
    for (node_id, node) in &graph.nodes {
        match &node.type_ {
            NodeType::TermVariable => {
                if let Some(term_var) = &node.term_variable {
                    if !existing_nodes.contains(&term_var.definition_node_id) {
                        errors.push(LintError {
                            code: LintErrorCode::IntegrityError.as_str().to_string(),
                            message: format!(
                                "Variable references non-existent definition node: {}",
                                term_var.definition_node_id
                            ),
                            node_id: *node_id,
                            location: get_source_location(graph, *node_id),
                        });
                    }
                }
            },
            NodeType::TermLambda => {
                if let Some(lambda) = &node.term_lambda {
                    if !existing_nodes.contains(&lambda.binder_variable_node_id) {
                        errors.push(LintError {
                            code: LintErrorCode::IntegrityError.as_str().to_string(),
                            message: format!(
                                "Lambda references non-existent binder variable: {}",
                                lambda.binder_variable_node_id
                            ),
                            node_id: *node_id,
                            location: get_source_location(graph, *node_id),
                        });
                    }
                    
                    if !existing_nodes.contains(&lambda.body_node_id) {
                        errors.push(LintError {
                            code: LintErrorCode::IntegrityError.as_str().to_string(),
                            message: format!(
                                "Lambda references non-existent body node: {}",
                                lambda.body_node_id
                            ),
                            node_id: *node_id,
                            location: get_source_location(graph, *node_id),
                        });
                    }
                    
                    if let Some(type_id) = lambda.type_annotation_id {
                        if !existing_nodes.contains(&type_id) {
                            errors.push(LintError {
                                code: LintErrorCode::IntegrityError.as_str().to_string(),
                                message: format!(
                                    "Lambda references non-existent type annotation: {}",
                                    type_id
                                ),
                                node_id: *node_id,
                                location: get_source_location(graph, *node_id),
                            });
                        }
                    }
                }
            },
            NodeType::TermApplication => {
                if let Some(app) = &node.term_application {
                    if !existing_nodes.contains(&app.function_node_id) {
                        errors.push(LintError {
                            code: LintErrorCode::IntegrityError.as_str().to_string(),
                            message: format!(
                                "Application references non-existent function node: {}",
                                app.function_node_id
                            ),
                            node_id: *node_id,
                            location: get_source_location(graph, *node_id),
                        });
                    }
                    
                    if !existing_nodes.contains(&app.argument_node_id) {
                        errors.push(LintError {
                            code: LintErrorCode::IntegrityError.as_str().to_string(),
                            message: format!(
                                "Application references non-existent argument node: {}",
                                app.argument_node_id
                            ),
                            node_id: *node_id,
                            location: get_source_location(graph, *node_id),
                        });
                    }
                }
            },
            // Add similar checks for other node types that reference other nodes
            NodeType::TermRef => {
                if let Some(term_ref) = &node.term_ref {
                    if !existing_nodes.contains(&term_ref.init_value_node_id) {
                        errors.push(LintError {
                            code: LintErrorCode::IntegrityError.as_str().to_string(),
                            message: format!(
                                "Ref references non-existent init value node: {}",
                                term_ref.init_value_node_id
                            ),
                            node_id: *node_id,
                            location: get_source_location(graph, *node_id),
                        });
                    }
                }
            },
            NodeType::TermDeref => {
                if let Some(term_deref) = &node.term_deref {
                    if !existing_nodes.contains(&term_deref.ref_node_id) {
                        errors.push(LintError {
                            code: LintErrorCode::IntegrityError.as_str().to_string(),
                            message: format!(
                                "Deref references non-existent ref node: {}",
                                term_deref.ref_node_id
                            ),
                            node_id: *node_id,
                            location: get_source_location(graph, *node_id),
                        });
                    }
                }
            },
            NodeType::TermAssign => {
                if let Some(term_assign) = &node.term_assign {
                    if !existing_nodes.contains(&term_assign.ref_node_id) {
                        errors.push(LintError {
                            code: LintErrorCode::IntegrityError.as_str().to_string(),
                            message: format!(
                                "Assignment references non-existent ref node: {}",
                                term_assign.ref_node_id
                            ),
                            node_id: *node_id,
                            location: get_source_location(graph, *node_id),
                        });
                    }
                    
                    if !existing_nodes.contains(&term_assign.value_node_id) {
                        errors.push(LintError {
                            code: LintErrorCode::IntegrityError.as_str().to_string(),
                            message: format!(
                                "Assignment references non-existent value node: {}",
                                term_assign.value_node_id
                            ),
                            node_id: *node_id,
                            location: get_source_location(graph, *node_id),
                        });
                    }
                }
            },
            // Add checks for other node types as needed
            _ => {} // Skip node types without references
        }
    }
}

/// Check variable scoping (TermVariable nodes link to valid binders)
fn check_variable_scopes(graph: &AsgGraph, errors: &mut Vec<LintError>) {
    // For a basic version, we'll do a simple traversal of the graph from the root
    // to build a scope map, then check each variable reference against it.
    
    // This is a simplified implementation - a real one would need to handle
    // scoping more carefully with proper lexical scope tracking
    
    // Map from variable node ID to its definition (binder) node ID
    let mut scope_map = HashMap::new();
    
    // Collect variable definitions from lambdas
    for (node_id, node) in &graph.nodes {
        if let NodeType::TermLambda = node.type_ {
            if let Some(lambda) = &node.term_lambda {
                // Lambda's binder variable is defined by this lambda
                scope_map.insert(lambda.binder_variable_node_id, *node_id);
            }
        }
    }
    
    // Check variable references
    for (node_id, node) in &graph.nodes {
        if let NodeType::TermVariable = node.type_ {
            if let Some(var) = &node.term_variable {
                let definition_id = var.definition_node_id;
                
                // For Level 0, we just check that the definition node exists and refers to a lambda
                if let Some(def_node) = graph.nodes.get(&definition_id) {
                    match def_node.type_ {
                        NodeType::TermLambda => {
                            // This is valid - variable refers to a lambda binder
                        },
                        _ => {
                            // Definition node is not a lambda - this might be valid in a more
                            // sophisticated scope system (e.g., let bindings), but for Level 0
                            // we'll flag it
                            errors.push(LintError {
                                code: LintErrorCode::ScopeError.as_str().to_string(),
                                message: format!(
                                    "Variable refers to non-binder node of type: {:?}",
                                    def_node.type_
                                ),
                                node_id: *node_id,
                                location: get_source_location(graph, *node_id),
                            });
                        }
                    }
                } else {
                    // This should be caught by check_node_references, but included for completeness
                    errors.push(LintError {
                        code: LintErrorCode::ScopeError.as_str().to_string(),
                        message: format!(
                            "Variable refers to non-existent definition node: {}",
                            definition_id
                        ),
                        node_id: *node_id,
                        location: get_source_location(graph, *node_id),
                    });
                }
            }
        }
    }
}

/// Check for simple type errors without full inference
fn check_simple_type_errors(graph: &AsgGraph, errors: &mut Vec<LintError>) {
    // This is a simplified implementation - a full type checker would be much more sophisticated
    
    for (node_id, node) in &graph.nodes {
        match node.type_ {
            NodeType::TermApplication => {
                if let Some(app) = &node.term_application {
                    // Check that the function being applied is a lambda (or at least could be)
                    if let Some(func_node) = graph.nodes.get(&app.function_node_id) {
                        match func_node.type_ {
                            NodeType::TermLambda => {
                                // Valid - applying a lambda
                            },
                            NodeType::TermVariable => {
                                // Could be valid - variables might refer to lambdas
                                // Full type inference would check this properly
                            },
                            NodeType::TermApplication => {
                                // Could be valid - an application might return a function
                                // Full type inference would check this properly
                            },
                            _ => {
                                // Applying something that's definitely not a function
                                errors.push(LintError {
                                    code: LintErrorCode::ApplicationError.as_str().to_string(),
                                    message: format!(
                                        "Applying non-function node of type: {:?}",
                                        func_node.type_
                                    ),
                                    node_id: *node_id,
                                    location: get_source_location(graph, *node_id),
                                });
                            }
                        }
                    }
                }
            },
            NodeType::TermAssign => {
                if let Some(assign) = &node.term_assign {
                    // Check that the target of assignment is a reference
                    if let Some(target_node) = graph.nodes.get(&assign.ref_node_id) {
                        match target_node.type_ {
                            NodeType::TermRef => {
                                // Valid - assigning to a ref
                            },
                            NodeType::TermDeref => {
                                // Valid - assigning through a deref
                            },
                            NodeType::TermVariable => {
                                // Could be valid - variables might refer to refs
                                // Full type inference would check this properly
                            },
                            _ => {
                                // Assigning to something that's definitely not a reference
                                errors.push(LintError {
                                    code: LintErrorCode::AssignmentError.as_str().to_string(),
                                    message: format!(
                                        "Assigning to non-reference node of type: {:?}",
                                        target_node.type_
                                    ),
                                    node_id: *node_id,
                                    location: get_source_location(graph, *node_id),
                                });
                            }
                        }
                    }
                }
            },
            _ => {} // No simple checks for other node types
        }
    }
}

/// Helper function to extract source location from metadata if available
fn get_source_location(graph: &AsgGraph, node_id: u64) -> Option<SourceLocation> {
    // Find any metadata nodes that might be associated with this node
    for (_, node) in &graph.nodes {
        if let NodeType::Metadata = node.type_ {
            if let Some(metadata) = &node.metadata {
                // Check if this metadata is for our node
                // In a real implementation, there would be a clearer link between nodes and their metadata
                
                if let Some(source_loc) = &metadata.source_location {
                    // In a real implementation, you'd check if this metadata applies to the node_id
                    // For now, we'll just return the first source location we find
                    return Some(SourceLocation {
                        filename: source_loc.filename.clone(),
                        start_line: source_loc.start_line,
                        start_col: source_loc.start_col,
                        end_line: source_loc.end_line,
                        end_col: source_loc.end_col,
                    });
                }
            }
        }
    }
    
    None
}