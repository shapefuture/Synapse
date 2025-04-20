//! Core IR structures for UPIR v2: System F polymorphism, ADTs, pattern matching.

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
}

#[derive(Debug, Clone)]
pub struct TypeParamDecl {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct DataTypeDecl {
    pub name: String,
    pub params: Vec<TypeParamDecl>,
    pub ctors: Vec<AdtConstructor>,
}

#[derive(Debug, Clone)]
pub struct AdtConstructor {
    pub name: String,
    pub field_types: Vec<TypeId>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub signature: FunctionSignature,
    pub type_params: Vec<TypeParamDecl>, // Extension: polymorphic
    pub regions: Vec<Region>,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub arg_types: Vec<TypeId>,
    pub result_types: Vec<TypeId>,
}

#[derive(Debug, Clone)]
pub struct Region {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: BlockId,
    pub arguments: Vec<BlockArgument>,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub name: String,
    pub operands: Vec<ValueId>,
    pub results: Vec<ValueDef>,
    pub attributes: HashMap<String, Attribute>,
    pub regions: Vec<Region>,

    // Extension: attach ADT and match metadata
    pub datatype_info: Option<DataTypeDecl>,
    pub match_info: Option<MatchInfo>,
}

#[derive(Debug, Clone)]
pub struct MatchInfo {
    pub arms: Vec<AdtMatchArm>,
}

#[derive(Debug, Clone)]
pub struct AdtMatchArm {
    pub ctor: String,
    pub vars: Vec<ValueId>,
    pub body_block: BlockId,
}

// ... BlockId, BlockArgument, ValueId, ValueDef, builder, etc, as previous ...