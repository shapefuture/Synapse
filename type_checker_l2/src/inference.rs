//! System F: Polymorphic type checking for TypeAbs, TypeApp nodes.

use super::types::Type;
use super::errors::{TypeError, Result};
use asg_core::{AsgGraph, NodeType};

use std::collections::HashMap;

pub struct TypeContext {
    /// term/expr variable environment: node_id → type
    pub vars: HashMap<u64, Type>,
    /// type variable environment: type_var_id → ()
    pub type_vars: HashMap<u64, ()>,
}

impl TypeContext {
    pub fn new() -> Self {
        TypeContext {
            vars: HashMap::new(),
            type_vars: HashMap::new(),
        }
    }
}

// Core recursive type inference.
pub fn infer(
    ctx: &mut TypeContext,
    node_id: u64,
    graph: &AsgGraph,
) -> Result<Type> {
    let node = graph.nodes.get(&node_id).ok_or(TypeError::UndefinedVariable(node_id))?;
    match node.type_ {
        NodeType::TypeAbs => {
            // ForAll (type lambda): extend type context, check body, wrap as ForAll
            let abs = node.type_abs.as_ref().ok_or(TypeError::Unimplemented)?;
            for &tvid in &abs.type_param_ids {
                ctx.type_vars.insert(tvid, ());
            }
            let body_ty = infer(ctx, abs.body_node_id, graph)?;
            for &tvid in &abs.type_param_ids {
                ctx.type_vars.remove(&tvid);
            }
            Ok(Type::ForAll(abs.type_param_ids.clone(), Box::new(body_ty)))
        }
        NodeType::TypeApp => {
            // Type application: instantiate ForAll type with provided arguments
            let tapp = node.type_app.as_ref().ok_or(TypeError::Unimplemented)?;
            let abs_ty = infer(ctx, tapp.type_abs_node_id, graph)?;
            if let Type::ForAll(params, body) = abs_ty {
                if params.len() != tapp.type_arg_node_ids.len() {
                    return Err(TypeError::UnificationFail(abs_ty.clone(), Type::Var(0)));
                }
                let mut subst = HashMap::new();
                for (i, &param_id) in params.iter().enumerate() {
                    let arg_ty = infer(ctx, tapp.type_arg_node_ids[i], graph)?;
                    subst.insert(param_id, arg_ty);
                }
                Ok(apply_type_subst(&body, &subst))
            } else {
                Err(TypeError::UnificationFail(abs_ty, Type::Var(1)))
            }
        }
        _ => Err(TypeError::Unimplemented)
    }
}

// Substitute type vars in t using subst map (type_var_id → Type)
fn apply_type_subst(t: &Type, subst: &HashMap<u64, Type>) -> Type {
    match t {
        Type::Int | Type::Bool => t.clone(),
        Type::Function(a, b) => Type::Function(
            Box::new(apply_type_subst(a, subst)), 
            Box::new(apply_type_subst(b, subst))),
        Type::Ref(inner) => Type::Ref(Box::new(apply_type_subst(inner, subst))),
        Type::Var(x) => subst.get(x).cloned().unwrap_or(t.clone()),
        Type::ForAll(vars, body) => {
            // Do not substitute bound vars
            let filtered: HashMap<u64, Type> = subst.iter()
                .filter(|(k,_)| !vars.contains(k)).map(|(k,v)|(k.clone(),v.clone())).collect();
            Type::ForAll(vars.clone(), Box::new(apply_type_subst(body, &filtered)))
        }
        Type::ADT(name, args) => Type::ADT(name.clone(), args.iter().map(|a| apply_type_subst(a, subst)).collect())
    }
}