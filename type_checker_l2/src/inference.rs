//! System F and ADT type checking for TypeAbs, TypeApp, DataDef, DataCtor, DataMatch.

use super::types::Type;
use super::errors::{TypeError, Result};
use asg_core::{AsgGraph, NodeType};

use std::collections::HashMap;

pub struct TypeContext {
    /// term/expr variable environment: node_id → type
    pub vars: HashMap<u64, Type>,
    /// type variable environment: type_var_id → ()
    pub type_vars: HashMap<u64, ()>,
    /// datatype definitions in scope: name → (param ids, ctors)
    pub datatypes: HashMap<String, (Vec<u64>, Vec<ADTConstructor>)>,
}

/// ADT constructor info for typechecking context.
#[derive(Debug, Clone)]
pub struct ADTConstructor {
    pub name: String,
    pub field_types: Vec<Type>,
}

impl TypeContext {
    pub fn new() -> Self {
        TypeContext {
            vars: HashMap::new(),
            type_vars: HashMap::new(),
            datatypes: HashMap::new(),
        }
    }
}

// ADT and System F style inference
pub fn infer(
    ctx: &mut TypeContext,
    node_id: u64,
    graph: &AsgGraph,
) -> Result<Type> {
    let node = graph.nodes.get(&node_id).ok_or(TypeError::UndefinedVariable(node_id))?;
    match node.type_ {
        NodeType::DataDef => {
            // Register type def in context.
            let dd = node.data_def.as_ref().ok_or(TypeError::Unimplemented)?;
            let ctors = dd.ctor_defs.iter().map(|ctor| {
                ADTConstructor {
                    name: ctor.name.clone(),
                    field_types: ctor.field_type_node_ids.iter()
                        .map(|&nid| infer(ctx, nid, graph).unwrap_or(Type::Var(nid)))
                        .collect(),
                }
            }).collect::<Vec<_>>();
            ctx.datatypes.insert(dd.name.clone(), (vec![], ctors));
            Ok(Type::ADT(dd.name.clone(), vec![]))
        }
        NodeType::DataCtor => {
            let ctor = node.data_ctor.as_ref().ok_or(TypeError::Unimplemented)?;
            // Find ADT type from context
            for (adt_name, (_, ctors)) in &ctx.datatypes {
                if let Some(found) = ctors.iter().find(|c| c.name == ctor.name) {
                    return Ok(Type::ADT(adt_name.clone(), found.field_types.clone()));
                }
            }
            Err(TypeError::Unimplemented)
        }
        NodeType::DataMatch => {
            let dm = node.data_match.as_ref().ok_or(TypeError::Unimplemented)?;
            let scrut_ty = infer(ctx, dm.scrutinee_node_id, graph)?;
            // All arms must produce the same type
            let mut res_type = None;
            for arm in &dm.arms {
                let arm_ty = infer(ctx, arm.body_node_id, graph)?;
                match &res_type {
                    Some(ty) if ty != &arm_ty => return Err(TypeError::UnificationFail(ty.clone(), arm_ty)),
                    None => res_type = Some(arm_ty),
                    _ => {}
                }
            }
            res_type.ok_or(TypeError::Unimplemented)
        }
        // --- System F nodes previously implemented ---
        NodeType::TypeAbs => {
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