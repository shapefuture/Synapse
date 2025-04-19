//! Algorithm W type inference for Synapse ASG.

use super::types::{Type, TypeScheme};
use super::unification::{SubstitutionMap, unify};
use super::errors::{TypeError, Result};

use asg_core::{AsgGraph, AsgNode, NodeType};
use std::collections::HashMap;

/// TypingContext maps variable node IDs to their type schemes (let-polymorphism).
pub struct TypingContext {
    // Maps ASG var node ID to type scheme
    pub bindings: HashMap<u64, TypeScheme>,
}

impl TypingContext {
    pub fn new() -> Self {
        TypingContext {
            bindings: HashMap::new(),
        }
    }

    /// Lookup type scheme of a variable, instantiating polymorphism
    pub fn lookup(&self, node_id: u64, fresh_counter: &mut u64) -> Result<Type> {
        let ts = self.bindings.get(&node_id).ok_or(TypeError::UndefinedVariable(node_id))?;
        Ok(instantiate(ts, fresh_counter))
    }

    /// Extend the typing context (shadowing) with a new binding
    pub fn extend(&mut self, node_id: u64, ty: TypeScheme) {
        self.bindings.insert(node_id, ty);
    }
}

/// Generalize a type into a scheme under the given context
pub fn generalize(ty: &Type, ctx: &TypingContext, subst: &SubstitutionMap) -> TypeScheme {
    // Find all free type variables in ty not present in ctx bindings
    let mut vars = Vec::new();
    get_free_type_vars(ty, &mut vars, subst);
    TypeScheme { vars, body: ty.clone() }
}

// Gets all free variables in the type and collects their IDs
fn get_free_type_vars(ty: &Type, vars: &mut Vec<u64>, subst: &SubstitutionMap) {
    match ty {
        Type::Var(id) => {
            let t = subst.get(id).unwrap_or(ty);
            if let Type::Var(id2) = t {
                if !vars.contains(id2) {
                    vars.push(*id2);
                }
            } else {
                get_free_type_vars(t, vars, subst);
            }
        }
        Type::Function(a, b) => {
            get_free_type_vars(a, vars, subst);
            get_free_type_vars(b, vars, subst);
        }
        Type::Ref(t) => get_free_type_vars(t, vars, subst),
        _ => {}
    }
}

// Instantiates a type scheme into a monotype by substituting fresh variables
fn instantiate(ts: &TypeScheme, fresh_counter: &mut u64) -> Type {
    let mut subst = HashMap::new();
    for &v in &ts.vars {
        subst.insert(v, Type::fresh_var(fresh_counter));
    }
    ts.body.apply(&subst)
}

/// The main type inference algorithm.
///
/// Returns the inferred type of the expression under context, updating the substitution map.
/// Recursively traverses the given node and subnodes.
pub fn infer(
    context: &mut TypingContext,
    node_id: u64,
    graph: &AsgGraph,
    subst: &mut SubstitutionMap,
    fresh_counter: &mut u64,
) -> Result<Type> {
    let node = graph.nodes.get(&node_id).ok_or(TypeError::UndefinedVariable(node_id))?;
    match node.type_ {
        NodeType::LiteralInt => Ok(Type::Int),
        NodeType::LiteralBool => Ok(Type::Bool),
        NodeType::TermVariable => {
            if let Some(var) = &node.term_variable {
                context.lookup(var.definition_node_id, fresh_counter)
            } else {
                Err(TypeError::UndefinedVariable(node_id))
            }
        },
        NodeType::TermLambda => {
            if let Some(lambda) = &node.term_lambda {
                let param_ty = Type::fresh_var(fresh_counter);
                let mut new_ctx = context.clone(); // Hide/shadow in inner context
                new_ctx.extend(lambda.binder_variable_node_id, TypeScheme { vars: vec![], body: param_ty.clone() });
                let body_ty = infer(&mut new_ctx, lambda.body_node_id, graph, subst, fresh_counter)?;
                Ok(Type::Function(Box::new(param_ty), Box::new(body_ty)))
            } else {
                Err(TypeError::Unimplemented)
            }
        },
        NodeType::TermApplication => {
            if let Some(app) = &node.term_application {
                let func_ty = infer(context, app.function_node_id, graph, subst, fresh_counter)?;
                let arg_ty = infer(context, app.argument_node_id, graph, subst, fresh_counter)?;
                let res_ty = Type::fresh_var(fresh_counter);
                unify(
                    &func_ty,
                    &Type::Function(Box::new(arg_ty), Box::new(res_ty.clone())),
                    subst,
                )?;
                Ok(res_ty.apply(subst))
            } else {
                Err(TypeError::Unimplemented)
            }
        },
        NodeType::PrimitiveOp => {
            // This example: only handle integer addition (+): arguments must have correct arity and types
            if let Some(op) = &node.primitive_op {
                match op.op_name.as_str() {
                    "+" | "-" | "*" | "/" => {
                        if op.argument_node_ids.len() != 2 {
                            return Err(TypeError::Unimplemented);
                        }
                        let lhs_ty = infer(context, op.argument_node_ids[0], graph, subst, fresh_counter)?;
                        let rhs_ty = infer(context, op.argument_node_ids[1], graph, subst, fresh_counter)?;
                        unify(&lhs_ty, &Type::Int, subst)?;
                        unify(&rhs_ty, &Type::Int, subst)?;
                        Ok(Type::Int)
                    }
                    _ => Err(TypeError::Unimplemented),
                }
            } else {
                Err(TypeError::Unimplemented)
            }
        }
        NodeType::TermRef => {
            if let Some(r) = &node.term_ref {
                let val_ty = infer(context, r.init_value_node_id, graph, subst, fresh_counter)?;
                Ok(Type::Ref(Box::new(val_ty)))
            } else {
                Err(TypeError::Unimplemented)
            }
        },
        NodeType::TermDeref => {
            if let Some(derefn) = &node.term_deref {
                let ref_ty = infer(context, derefn.ref_node_id, graph, subst, fresh_counter)?;
                match ref_ty {
                    Type::Ref(inner) => Ok(*inner),
                    _ => Err(TypeError::ApplicationMismatch(node_id)),
                }
            } else {
                Err(TypeError::Unimplemented)
            }
        },
        NodeType::TermAssign => {
            if let Some(assign) = &node.term_assign {
                let ref_ty = infer(context, assign.ref_node_id, graph, subst, fresh_counter)?;
                let val_ty = infer(context, assign.value_node_id, graph, subst, fresh_counter)?;
                match ref_ty {
                    Type::Ref(inner) => {
                        unify(&*inner, &val_ty, subst)?;
                        Ok(Type::Bool) // assignment returns bool (or unit, but defaulting to bool for now)
                    }
                    _ => Err(TypeError::ApplicationMismatch(node_id)),
                }
            } else {
                Err(TypeError::Unimplemented)
            }
        },
        // Metadata nodes, proof obligations, etc. are ignored
        _ => Err(TypeError::Unimplemented),
    }
}