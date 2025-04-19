//! Algorithm W type inference for Synapse ASG.

use super::types::{Type, TypeScheme};
use super::unification::SubstitutionMap;
use super::errors::{TypeError, Result};

use asg_core::{AsgGraph, AsgNode, NodeType};
use std::collections::HashMap;

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
}

pub fn infer(
    context: &mut TypingContext,
    node_id: u64,
    graph: &AsgGraph,
    subst: &mut SubstitutionMap,
    fresh_counter: &mut u64,
) -> Result<Type> {
    // Fetch the node
    let node = graph.nodes.get(&node_id).ok_or(TypeError::UndefinedVariable(node_id))?;
    match node.type_ {
        NodeType::LiteralInt => Ok(Type::Int),
        NodeType::LiteralBool => Ok(Type::Bool),
        NodeType::TermVariable => {
            // Look up variable in context (polymorphic)
            if let Some(var) = &node.term_variable {
                let ts = context.bindings.get(&var.definition_node_id)
                    .ok_or(TypeError::UndefinedVariable(var.definition_node_id))?;
                Ok(instantiate(ts, fresh_counter))
            } else {
                Err(TypeError::UndefinedVariable(node_id))
            }
        },
        NodeType::TermLambda => {
            // Lambda: create type variable for argument and infer the body
            if let Some(lambda) = &node.term_lambda {
                let param_ty = Type::fresh_var(fresh_counter);
                // Extend context for the binder variable
                context.bindings.insert(lambda.binder_variable_node_id, TypeScheme { vars: vec![], body: param_ty.clone() });
                let body_ty = infer(context, lambda.body_node_id, graph, subst, fresh_counter)?;
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
                super::unification::unify(
                    &func_ty,
                    &Type::Function(Box::new(arg_ty), Box::new(res_ty.clone())),
                    subst,
                )?;
                Ok(res_ty.apply(subst))
            } else {
                Err(TypeError::Unimplemented)
            }
        },
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
                        super::unification::unify(&*inner, &val_ty, subst)?;
                        Ok(Type::Bool) // assignment returns bool (or unit, but defaulting to bool for Phase 0)
                    }
                    _ => Err(TypeError::ApplicationMismatch(node_id)),
                }
            } else {
                Err(TypeError::Unimplemented)
            }
        },
        _ => Err(TypeError::Unimplemented),
    }
}

fn instantiate(ts: &TypeScheme, fresh_counter: &mut u64) -> Type {
    // Substitute type variables with fresh ones for let-polymorphism
    let mut subst = HashMap::new();
    for &v in &ts.vars {
        subst.insert(v, Type::fresh_var(fresh_counter));
    }
    ts.body.apply(&subst)
}