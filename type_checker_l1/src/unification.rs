//! Unification logic for Synapse Hindley-Milner type inference.

use super::types::Type;
use super::errors::{TypeError, Result};

use std::collections::HashMap;

pub type SubstitutionMap = HashMap<u64, Type>;

pub fn unify(_t1: &Type, _t2: &Type, _subst: &mut SubstitutionMap) -> Result<()> {
    // Stub
    Ok(())
}