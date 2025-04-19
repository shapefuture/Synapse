//! Core parser for the Synapse language.
//!
//! This library provides functionality to parse Synapse source code into an ASG.

pub mod ast;
pub mod error;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use asg_core::{
    AsgGraph, NodeType, TermVariable, TermLambda, TermApplication, 
    LiteralInt, LiteralBool, PrimitiveOp, TermRef, TermDeref, TermAssign,
    EffectPerform, TypeNode, TypeKind,
};

use error::{ParseError, Result};

// Include the generated parser
#[allow(clippy::all)]
#[rustfmt::skip]
#[allow(unused_imports)]
mod core_syntax {
    include!(concat!(env!("OUT_DIR"), "/core_syntax.rs"));
}

/// Parses a Synapse source file into an ASG.
pub fn parse_file<P: AsRef<Path>>(file_path: P) -> Result<AsgGraph> {
    let source = fs::read_to_string(&file_path)?;
    let file_name = file_path.as_ref().to_string_lossy().to_string();
    
    parse_source(&source, Some(file_name))
}

/// Parses Synapse source code into an ASG.
pub fn parse_source(source: &str, file_name: Option<String>) -> Result<AsgGraph> {
    // Parse into intermediate AST
    let parser = core_syntax::RootParser::new();
    let ast = parser.parse(source).map_err(ParseError::from)?;
    
    // Convert AST to ASG
    build_asg(ast)
}

/// Converts an AST to an ASG.
fn build_asg(ast: ast::Root) -> Result<AsgGraph> {
    let mut graph = AsgGraph::new();
    let mut builder = AsgBuilder::new(&mut graph);
    
    // Build the root expression
    let root_id = builder.build_expr(&ast.expr)?;
    graph.set_root(root_id)?;
    
    Ok(graph)
}

/// Helper for building an ASG from an AST.
struct AsgBuilder<'a> {
    /// The graph being built
    graph: &'a mut AsgGraph,
    /// Maps variable names to their definition node IDs
    name_to_def: HashMap<String, u64>,
    /// Maps AST types to ASG type node IDs
    type_cache: HashMap<String, u64>,
}

impl<'a> AsgBuilder<'a> {
    fn new(graph: &'a mut AsgGraph) -> Self {
        Self {
            graph,
            name_to_def: HashMap::new(),
            type_cache: HashMap::new(),
        }
    }
    
    /// Builds an ASG expression from an AST expression.
    fn build_expr(&mut self, expr: &ast::Expr) -> Result<u64> {
        match expr {
            ast::Expr::Variable(name) => self.build_variable(name),
            ast::Expr::Lambda(param, ty, body) => self.build_lambda(param, ty.as_ref(), body),
            ast::Expr::Application(func, arg) => self.build_application(func, arg),
            ast::Expr::IntLiteral(value) => self.build_int_literal(*value),
            ast::Expr::BoolLiteral(value) => self.build_bool_literal(*value),
            ast::Expr::PrimitiveOp(op, args) => self.build_primitive_op(op, args),
            ast::Expr::Ref(expr) => self.build_ref(expr),
            ast::Expr::Deref(expr) => self.build_deref(expr),
            ast::Expr::Assign(lhs, rhs) => self.build_assign(lhs, rhs),
            ast::Expr::Perform(effect, expr) => self.build_perform(effect, expr),
        }
    }
    
    /// Builds a variable reference.
    fn build_variable(&self, name: &str) -> Result<u64> {
        let def_id = self.name_to_def.get(name).cloned().unwrap_or(0);
        
        let node_id = self.graph.add_node(
            NodeType::TERM_VARIABLE,
            TermVariable {
                name: name.to_string(),
                definition_node_id: def_id,
            },
        );
        
        Ok(node_id)
    }
    
    /// Builds a lambda abstraction.
    fn build_lambda(&mut self, param: &str, ty: Option<&ast::Type>, body: &ast::Expr) -> Result<u64> {
        // Create a new scope for this lambda
        let mut inner_builder = AsgBuilder {
            graph: self.graph,
            name_to_def: self.name_to_def.clone(),
            type_cache: self.type_cache.clone(),
        };
        
        // Build parameter variable
        let param_id = inner_builder.graph.add_node(
            NodeType::TERM_VARIABLE,
            TermVariable {
                name: param.to_string(),
                definition_node_id: 0, // Will be updated after creating the lambda
            },
        );
        
        // Build optional type annotation
        let type_id = if let Some(ty) = ty {
            inner_builder.build_type(ty)?
        } else {
            0 // No type annotation
        };
        
        // Add the parameter to the scope
        inner_builder.name_to_def.insert(param.to_string(), param_id);
        
        // Build the body in the inner scope
        let body_id = inner_builder.build_expr(body)?;
        
        // Create the lambda node
        let lambda_id = inner_builder.graph.add_node(
            NodeType::TERM_LAMBDA,
            TermLambda {
                binder_variable_node_id: param_id,
                body_node_id: body_id,
                type_annotation_id: type_id,
            },
        );
        
        // Update the parameter's definition to point to the lambda
        if let Some(node) = inner_builder.graph.get_node_mut(param_id) {
            if let Some(asg_core::generated::asg_node::Content::TermVariable(ref mut var)) = node.content {
                var.definition_node_id = lambda_id;
            }
        }
        
        // Update our state from the inner builder
        self.type_cache = inner_builder.type_cache;
        
        Ok(lambda_id)
    }
    
    /// Builds a function application.
    fn build_application(&mut self, func: &ast::Expr, arg: &ast::Expr) -> Result<u64> {
        let func_id = self.build_expr(func)?;
        let arg_id = self.build_expr(arg)?;
        
        let app_id = self.graph.add_node(
            NodeType::TERM_APPLICATION,
            TermApplication {
                function_node_id: func_id,
                argument_node_id: arg_id,
            },
        );
        
        Ok(app_id)
    }
    
    /// Builds an integer literal.
    fn build_int_literal(&mut self, value: i64) -> Result<u64> {
        let node_id = self.graph.add_node(
            NodeType::LITERAL_INT,
            LiteralInt { value },
        );
        
        Ok(node_id)
    }
    
    /// Builds a boolean literal.
    fn build_bool_literal(&mut self, value: bool) -> Result<u64> {
        let node_id = self.graph.add_node(
            NodeType::LITERAL_BOOL,
            LiteralBool { value },
        );
        
        Ok(node_id)
    }
    
    /// Builds a primitive operation.
    fn build_primitive_op(&mut self, op: &str, args: &[ast::Expr]) -> Result<u64> {
        let mut arg_ids = Vec::with_capacity(args.len());
        
        for arg in args {
            let arg_id = self.build_expr(arg)?;
            arg_ids.push(arg_id);
        }
        
        let node_id = self.graph.add_node(
            NodeType::PRIMITIVE_OP,
            PrimitiveOp {
                op_name: op.to_string(),
                argument_node_ids: arg_ids,
            },
        );
        
        Ok(node_id)
    }
    
    /// Builds a reference creation.
    fn build_ref(&mut self, expr: &ast::Expr) -> Result<u64> {
        let expr_id = self.build_expr(expr)?;
        
        let node_id = self.graph.add_node(
            NodeType::TERM_REF,
            TermRef {
                init_value_node_id: expr_id,
            },
        );
        
        Ok(node_id)
    }
    
    /// Builds a dereference operation.
    fn build_deref(&mut self, expr: &ast::Expr) -> Result<u64> {
        let expr_id = self.build_expr(expr)?;
        
        let node_id = self.graph.add_node(
            NodeType::TERM_DEREF,
            TermDeref {
                ref_node_id: expr_id,
            },
        );
        
        Ok(node_id)
    }
    
    /// Builds an assignment operation.
    fn build_assign(&mut self, lhs: &ast::Expr, rhs: &ast::Expr) -> Result<u64> {
        let lhs_id = self.build_expr(lhs)?;
        let rhs_id = self.build_expr(rhs)?;
        
        let node_id = self.graph.add_node(
            NodeType::TERM_ASSIGN,
            TermAssign {
                ref_node_id: lhs_id,
                value_node_id: rhs_id,
            },
        );
        
        Ok(node_id)
    }
    
    /// Builds an effect performance.
    fn build_perform(&mut self, effect: &str, expr: &ast::Expr) -> Result<u64> {
        let expr_id = self.build_expr(expr)?;
        
        let node_id = self.graph.add_node(
            NodeType::EFFECT_PERFORM,
            EffectPerform {
                effect_name: effect.to_string(),
                value_node_id: expr_id,
            },
        );
        
        Ok(node_id)
    }
    
    /// Builds a type node from an AST type.
    fn build_type(&mut self, ty: &ast::Type) -> Result<u64> {
        // Check cache first
        let key = ty.to_string();
        if let Some(&id) = self.type_cache.get(&key) {
            return Ok(id);
        }
        
        // Build the type node
        let type_id = match ty {
            ast::Type::Int => {
                self.graph.add_node(
                    NodeType::TYPE_NODE,
                    TypeNode {
                        node_id: 0, // Will be overwritten
                        type_kind: TypeKind::TYPE_INT as i32,
                        content: Some(asg_core::generated::type_node::Content::TypeInt(
                            asg_core::generated::TypeInt {},
                        )),
                    },
                )
            }
            ast::Type::Bool => {
                self.graph.add_node(
                    NodeType::TYPE_NODE,
                    TypeNode {
                        node_id: 0, // Will be overwritten
                        type_kind: TypeKind::TYPE_BOOL as i32,
                        content: Some(asg_core::generated::type_node::Content::TypeBool(
                            asg_core::generated::TypeBool {},
                        )),
                    },
                )
            }
            ast::Type::Function(param_ty, return_ty) => {
                let param_id = self.build_type(param_ty)?;
                let return_id = self.build_type(return_ty)?;
                
                self.graph.add_node(
                    NodeType::TYPE_NODE,
                    TypeNode {
                        node_id: 0, // Will be overwritten
                        type_kind: TypeKind::TYPE_FUNCTION as i32,
                        content: Some(asg_core::generated::type_node::Content::TypeFunction(
                            asg_core::generated::TypeFunction {
                                parameter_type_id: param_id,
                                return_type_id: return_id,
                            },
                        )),
                    },
                )
            }
            ast::Type::Ref(element_ty) => {
                let element_id = self.build_type(element_ty)?;
                
                self.graph.add_node(
                    NodeType::TYPE_NODE,
                    TypeNode {
                        node_id: 0, // Will be overwritten
                        type_kind: TypeKind::TYPE_REF as i32,
                        content: Some(asg_core::generated::type_node::Content::TypeRef(
                            asg_core::generated::TypeRef {
                                element_type_id: element_id,
                            },
                        )),
                    },
                )
            }
            ast::Type::Unit => {
                self.graph.add_node(
                    NodeType::TYPE_NODE,
                    TypeNode {
                        node_id: 0, // Will be overwritten
                        type_kind: TypeKind::TYPE_UNIT as i32,
                        content: Some(asg_core::generated::type_node::Content::TypeUnit(
                            asg_core::generated::TypeUnit {},
                        )),
                    },
                )
            }
        };
        
        // Fix the node_id in the TypeNode
        if let Some(node) = self.graph.get_node_mut(type_id) {
            if let Some(asg_core::generated::asg_node::Content::TypeNode(ref mut type_node)) = node.content {
                type_node.node_id = type_id;
            }
        }
        
        // Add to cache
        self.type_cache.insert(key, type_id);
        
        Ok(type_id)
    }
}