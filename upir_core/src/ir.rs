//! UPIR v2: propagate effect tags on operations and modules.

use crate::types::*;
use crate::attributes::*;
use crate::dialects::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub functions: Vec<Function>,
    pub datatype_decls: Vec<DataTypeDecl>,
    pub typeparam_decls: Vec<TypeParamDecl>,
    pub effect_decls: Vec<EffectDecl>,
    pub effect_tags: Vec<String>, // new: module-level
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub name: String,
    pub operands: Vec<ValueId>,
    pub results: Vec<ValueDef>,
    pub attributes: HashMap<String, Attribute>,
    pub regions: Vec<Region>,
    pub datatype_info: Option<DataTypeDecl>,
    pub match_info: Option<MatchInfo>,
    pub effect_tags: Vec<String>, // new: op-level
}
// ... rest as before ...