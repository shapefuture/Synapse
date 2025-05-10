//! Core formatter for the Synapse language.
//!
//! This library provides functionality to convert an ASG back into human-readable text.

use std::collections::HashSet;

use asg_core::{
    AsgGraph, NodeType, TermVariable, TermLambda, TermApplication, 
    LiteralInt, LiteralBool, PrimitiveOp, TermRef, TermDeref, TermAssign,
    EffectPerform, TypeNode, TypeKind,
};

/// Error type for formatting operations.
#[derive(thiserror::Error, Debug)]
pub enum FormatError {
    #[error("Node not found: {0}")]
    NodeNotFound(u64),
    
    #[error("Invalid node type: expected {expected}, found {found}")]
    InvalidNodeType {
        expected: String,
        found: String,
    },
    
    #[error("Cycle detected in ASG")]
    CycleDetected,
    
    #[error("Unknown node type: {0}")]
    UnknownNodeType(i32),
    
    #[error("ASG Error: {0}")]
    AsgError(#[from] asg_core::Error),
}

/// Result type for formatting operations.
pub type Result<T> = std::result::Result<T, FormatError>;

/// Formats an ASG into human-readable text.
///
/// This function converts the ASG back into the minimal concrete syntax
/// for the Synapse language, starting from the given root node.
pub fn format_asg(graph: &AsgGraph, root_id: u64) -> Result<String> {
    let mut formatter = PrettyPrinter::new(graph);
    formatter.format_node(root_id)
}

/// Helper struct for pretty-printing an ASG.
struct PrettyPrinter<'a> {
    /// The graph being formatted
    graph: &'a AsgGraph,
    /// Set of nodes being visited (to detect cycles)
    visiting: HashSet<u64>,
}

impl<'a> PrettyPrinter<'a> {
    fn new(graph: &'a AsgGraph) -> Self {
        Self {
            graph,
            visiting: HashSet::new(),
        }
    }
    
    /// Formats a node into a string.
    fn format_node(&mut self, node_id: u64) -> Result<String> {
        // Check for cycles
        if !self.visiting.insert(node_id) {
            return Err(FormatError::CycleDetected);
        }
        
        let node = self.graph.get_node(node_id)
            .ok_or(FormatError::NodeNotFound(node_id))?;
        
        let result = match node.r#type {
            typ if typ == NodeType::TERM_VARIABLE as i32 => self.format_variable(node_id)?,
            typ if typ == NodeType::TERM_LAMBDA as i32 => self.format_lambda(node_id)?,
            typ if typ == NodeType::TERM_APPLICATION as i32 => self.format_application(node_id)?,
            typ if typ == NodeType::LITERAL_INT as i32 => self.format_int_literal(node_id)?,
            typ if typ == NodeType::LITERAL_BOOL as i32 => self.format_bool_literal(node_id)?,
            typ if typ == NodeType::PRIMITIVE_OP as i32 => self.format_primitive_op(node_id)?,
            typ if typ == NodeType::TERM_REF as i32 => self.format_ref(node_id)?,
            typ if typ == NodeType::TERM_DEREF as i32 => self.format_deref(node_id)?,
            typ if typ == NodeType::TERM_ASSIGN as i32 => self.format_assign(node_id)?,
            typ if typ == NodeType::EFFECT_PERFORM as i32 => self.format_perform(node_id)?,
            typ if typ == NodeType::TYPE_NODE as i32 => self.format_type(node_id)?,
            _ => return Err(FormatError::UnknownNodeType(node.r#type)),
        };
        
        // Remove from visiting set
        self.visiting.remove(&node_id);
        
        Ok(result)
    }
    
    /// Formats a variable node.
    fn format_variable(&mut self, node_id: u64) -> Result<String> {
        let var = self.graph.get_variable(node_id)
            .map_err(FormatError::from)?;
        
        Ok(var.name.clone())
    }
    
    /// Formats a lambda node.
    fn format_lambda(&mut self, node_id: u64) -> Result<String> {
        let lambda = self.graph.get_lambda(node_id)
            .map_err(FormatError::from)?;
        
        // Format the parameter
        let param_node = self.graph.get_node(lambda.binder_variable_node_id)
            .ok_or(FormatError::NodeNotFound(lambda.binder_variable_node_id))?;
        
        let param_name = if let Some(asg_core::generated::asg_node::Content::TermVariable(var)) = &param_node.content {
            var.name.clone()
        } else {
            return Err(FormatError::InvalidNodeType {
                expected: "TermVariable".to_string(),
                found: format!("{:?}", param_node.content),
            });
        };
        
        // Format the optional type annotation
        let type_annotation = if lambda.type_annotation_id != 0 {
            format!(": {}", self.format_type(lambda.type_annotation_id)?)
        } else {
            "".to_string()
        };
        
        // Format the body
        let body = self.format_node(lambda.body_node_id)?;
        
        Ok(format!("({}{}) => {}", param_name, type_annotation, body))
    }
    
    /// Formats an application node.
    fn format_application(&mut self, node_id: u64) -> Result<String> {
        let app = self.graph.get_application(node_id)
            .map_err(FormatError::from)?;
        
        let func = self.format_node(app.function_node_id)?;
        let arg = self.format_node(app.argument_node_id)?;
        
        // Add parentheses to ensure correct precedence
        let func_with_parens = if self.needs_parens_in_application_func(app.function_node_id) {
            format!("({})", func)
        } else {
            func
        };
        
        Ok(format!("{}({})", func_with_parens, arg))
    }
    
    /// Checks if a function node needs parentheses when used in application context.
    fn needs_parens_in_application_func(&self, node_id: u64) -> bool {
        if let Some(node) = self.graph.get_node(node_id) {
            // Lambda and low-precedence operations need parentheses
            node.r#type == NodeType::TERM_LAMBDA as i32 ||
            node.r#type == NodeType::TERM_ASSIGN as i32
        } else {
            false
        }
    }
    
    /// Formats an integer literal.
    fn format_int_literal(&mut self, node_id: u64) -> Result<String> {
        let node = self.graph.get_node(node_id)
            .ok_or(FormatError::NodeNotFound(node_id))?;
        
        if let Some(asg_core::generated::asg_node::Content::LiteralInt(lit)) = &node.content {
            Ok(lit.value.to_string())
        } else {
            Err(FormatError::InvalidNodeType {
                expected: "LiteralInt".to_string(),
                found: format!("{:?}", node.content),
            })
        }
    }
    
    /// Formats a boolean literal.
    fn format_bool_literal(&mut self, node_id: u64) -> Result<String> {
        let node = self.graph.get_node(node_id)
            .ok_or(FormatError::NodeNotFound(node_id))?;
        
        if let Some(asg_core::generated::asg_node::Content::LiteralBool(lit)) = &node.content {
            Ok(if lit.value { "true".to_string() } else { "false".to_string() })
        } else {
            Err(FormatError::InvalidNodeType {
                expected: "LiteralBool".to_string(),
                found: format!("{:?}", node.content),
            })
        }
    }
    
    /// Formats a primitive operation.
    fn format_primitive_op(&mut self, node_id: u64) -> Result<String> {
        let node = self.graph.get_node(node_id)
            .ok_or(FormatError::NodeNotFound(node_id))?;
        
        if let Some(asg_core::generated::asg_node::Content::PrimitiveOp(op)) = &node.content {
            // Format the arguments
            let mut arg_strs = Vec::with_capacity(op.argument_node_ids.len());
            for &arg_id in &op.argument_node_ids {
                arg_strs.push(self.format_node(arg_id)?);
            }
            
            // Special case for binary operators with infix notation
            if op.argument_node_ids.len() == 2 {
                match op.op_name.as_str() {
                    "add" => return Ok(format!("{} + {}", arg_strs[0], arg_strs[1])),
                    "sub" => return Ok(format!("{} - {}", arg_strs[0], arg_strs[1])),
                    "mul" => return Ok(format!("{} * {}", arg_strs[0], arg_strs[1])),
                    "div" => return Ok(format!("{} / {}", arg_strs[0], arg_strs[1])),
                    "mod" => return Ok(format!("{} % {}", arg_strs[0], arg_strs[1])),
                    "eq" => return Ok(format!("{} == {}", arg_strs[0], arg_strs[1])),
                    "neq" => return Ok(format!("{} != {}", arg_strs[0], arg_strs[1])),
                    "lt" => return Ok(format!("{} < {}", arg_strs[0], arg_strs[1])),
                    "gt" => return Ok(format!("{} > {}", arg_strs[0], arg_strs[1])),
                    "lte" => return Ok(format!("{} <= {}", arg_strs[0], arg_strs[1])),
                    "gte" => return Ok(format!("{} >= {}", arg_strs[0], arg_strs[1])),
                    "and" => return Ok(format!("{} && {}", arg_strs[0], arg_strs[1])),
                    "or" => return Ok(format!("{} || {}", arg_strs[0], arg_strs[1])),
                    _ => {}
                }
            }
            
            // Generic format for other operations
            Ok(format!("{}({})", op.op_name, arg_strs.join(", ")))
        } else {
            Err(FormatError::InvalidNodeType {
                expected: "PrimitiveOp".to_string(),
                found: format!("{:?}", node.content),
            })
        }
    }
    
    /// Formats a reference creation.
    fn format_ref(&mut self, node_id: u64) -> Result<String> {
        let node = self.graph.get_node(node_id)
            .ok_or(FormatError::NodeNotFound(node_id))?;
        
        if let Some(asg_core::generated::asg_node::Content::TermRef(ref_)) = &node.content {
            let value = self.format_node(ref_.init_value_node_id)?;
            
            // Add parentheses if needed
            let value_with_parens = if self.needs_parens_in_unary_arg(ref_.init_value_node_id) {
                format!("({})", value)
            } else {
                value
            };
            
            Ok(format!("ref {}", value_with_parens))
        } else {
            Err(FormatError::InvalidNodeType {
                expected: "TermRef".to_string(),
                found: format!("{:?}", node.content),
            })
        }
    }
    
    /// Formats a dereference operation.
    fn format_deref(&mut self, node_id: u64) -> Result<String> {
        let node = self.graph.get_node(node_id)
            .ok_or(FormatError::NodeNotFound(node_id))?;
        
        if let Some(asg_core::generated::asg_node::Content::TermDeref(deref)) = &node.content {
            let value = self.format_node(deref.ref_node_id)?;
            
            // Add parentheses if needed
            let value_with_parens = if self.needs_parens_in_unary_arg(deref.ref_node_id) {
                format!("({})", value)
            } else {
                value
            };
            
            Ok(format!("!{}", value_with_parens))
        } else {
            Err(FormatError::InvalidNodeType {
                expected: "TermDeref".to_string(),
                found: format!("{:?}", node.content),
            })
        }
    }
    
    /// Checks if a node needs parentheses when used as an argument to a unary operator.
    fn needs_parens_in_unary_arg(&self, node_id: u64) -> bool {
        if let Some(node) = self.graph.get_node(node_id) {
            // Binary operations and other non-atomic expressions need parentheses
            node.r#type == NodeType::PRIMITIVE_OP as i32 ||
            node.r#type == NodeType::TERM_ASSIGN as i32 ||
            node.r#type == NodeType::TERM_LAMBDA as i32
        } else {
            false
        }
    }
    
    /// Formats an assignment operation.
    fn format_assign(&mut self, node_id: u64) -> Result<String> {
        let node = self.graph.get_node(node_id)
            .ok_or(FormatError::NodeNotFound(node_id))?;
        
        if let Some(asg_core::generated::asg_node::Content::TermAssign(assign)) = &node.content {
            let lhs = self.format_node(assign.ref_node_id)?;
            let rhs = self.format_node(assign.value_node_id)?;
            
            Ok(format!("{} := {}", lhs, rhs))
        } else {
            Err(FormatError::InvalidNodeType {
                expected: "TermAssign".to_string(),
                found: format!("{:?}", node.content),
            })
        }
    }
    
    /// Formats an effect performance.
    fn format_perform(&mut self, node_id: u64) -> Result<String> {
        let node = self.graph.get_node(node_id)
            .ok_or(FormatError::NodeNotFound(node_id))?;
        
        if let Some(asg_core::generated::asg_node::Content::EffectPerform(perform)) = &node.content {
            let value = self.format_node(perform.value_node_id)?;
            
            Ok(format!("perform('{}', {})", perform.effect_name, value))
        } else {
            Err(FormatError::InvalidNodeType {
                expected: "EffectPerform".to_string(),
                found: format!("{:?}", node.content),
            })
        }
    }
    
    /// Formats a type node.
    fn format_type(&mut self, node_id: u64) -> Result<String> {
        let node = self.graph.get_node(node_id)
            .ok_or(FormatError::NodeNotFound(node_id))?;
        
        if let Some(asg_core::generated::asg_node::Content::TypeNode(ty)) = &node.content {
            match ty.type_kind {
                kind if kind == TypeKind::TYPE_INT as i32 => Ok("Int".to_string()),
                kind if kind == TypeKind::TYPE_BOOL as i32 => Ok("Bool".to_string()),
                kind if kind == TypeKind::TYPE_UNIT as i32 => Ok("Unit".to_string()),
                kind if kind == TypeKind::TYPE_FUNCTION as i32 => {
                    if let Some(asg_core::generated::type_node::Content::TypeFunction(func)) = &ty.content {
                        let param = self.format_type(func.parameter_type_id)?;
                        let ret = self.format_type(func.return_type_id)?;
                        
                        Ok(format!("{} -> {}", param, ret))
                    } else {
                        Err(FormatError::InvalidNodeType {
                            expected: "TypeFunction".to_string(),
                            found: format!("{:?}", ty.content),
                        })
                    }
                },
                kind if kind == TypeKind::TYPE_REF as i32 => {
                    if let Some(asg_core::generated::type_node::Content::TypeRef(ref_)) = &ty.content {
                        let elem = self.format_type(ref_.element_type_id)?;
                        
                        Ok(format!("Ref {}", elem))
                    } else {
                        Err(FormatError::InvalidNodeType {
                            expected: "TypeRef".to_string(),
                            found: format!("{:?}", ty.content),
                        })
                    }
                },
                _ => Err(FormatError::UnknownNodeType(ty.type_kind)),
            }
        } else {
            Err(FormatError::InvalidNodeType {
                expected: "TypeNode".to_string(),
                found: format!("{:?}", node.content),
            })
        }
    }
}