//! Unification logic for Synapse Hindley-Milner type inference.

use super::types::Type;
use super::errors::{TypeError, Result};

use std::collections::HashMap;

pub type SubstitutionMap = HashMap<u64, Type>;

pub fn unify(t1: &Type, t2: &Type, subst: &mut SubstitutionMap) -> Result<()> {
    match (t1, t2) {
        (Type::Int, Type::Int) | (Type::Bool, Type::Bool) => Ok(()),
        (Type::Ref(a), Type::Ref(b)) => unify(a, b, subst),
        (Type::Function(a1, b1), Type::Function(a2, b2)) => {
            unify(a1, a2, subst)?;
            unify(b1, b2, subst)
        }
        (Type::Var(v), t) | (t, Type::Var(v)) => bind_var(*v, t, subst),
        _ => Err(TypeError::UnificationFail(t1.clone(), t2.clone())),
    }
}

fn bind_var(var: u64, t: &Type, subst: &mut SubstitutionMap) -> Result<()> {
    if let Type::Var(v2) = t {
        if var == *v2 {
            return Ok(());
        }
    }
    if occurs_check(var, t, subst) {
        return Err(TypeError::OccursCheck);
    }
    subst.insert(var, t.clone());
    Ok(())
}

fn occurs_check(var: u64, t: &Type, subst: &SubstitutionMap) -> bool {
    match t {
        Type::Var(v) => {
            if *v == var {
                true
            } else if let Some(t2) = subst.get(v) {
                occurs_check(var, t2, subst)
            } else {
                false
            }
        }
        Type::Function(a, b) => occurs_check(var, a, subst) || occurs_check(var, b, subst),
        Type::Ref(t1) => occurs_check(var, t1, subst),
        _ => false,
    }
}