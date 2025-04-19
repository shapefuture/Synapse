//! Core IR structures for UPIR, v2: ADTs, polymorphism, effects extension points.

use crate::types::*;
use crate::attributes::*;
use crate::dialects::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub functions: Vec<Function>,
    /// Extension: ADT, type, and effect definitions can be stored at module level.
    pub datatype_decls: Vec<DataTypeDecl>,
    pub typeparam_decls: Vec<TypeParamDecl>,
    pub effect_decls: Vec<EffectDecl>,
}

#[derive(Debug, Clone)]
pub struct DataTypeDecl {
    pub name: String,
    pub params: Vec<TypeParamDecl>,
    pub ctors: Vec<AdtConstructor>,
}

#[derive(Debug, Clone)]
pub struct TypeParamDecl {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct AdtConstructor {
    pub name: String,
    pub field_types: Vec<TypeId>,
}

#[derive(Debug, Clone)]
pub struct EffectDecl {
    pub name: String,
    pub info: String,
}

// ... rest of ir.rs as in previous version ...