//! Core IR structures for UPIR.

use crate::types::*;
use crate::attributes::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub signature: FunctionSignature,
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
    pub name: String, // e.g., "core.add"
    pub operands: Vec<ValueId>,
    pub results: Vec<ValueDef>,
    pub attributes: HashMap<String, Attribute>,
    pub regions: Vec<Region>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u64);

#[derive(Debug, Clone)]
pub struct BlockArgument {
    pub value_def: ValueDef,
    pub block_id: BlockId,
    pub index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueId(pub u64);

#[derive(Debug, Clone)]
pub struct ValueDef {
    pub id: ValueId,
    pub ty: TypeId,
}

/// Builder pattern for safe construction (stub for extension).
pub struct IRBuilder;

impl IRBuilder {
    pub fn new() -> Self { Self }
}

/// Pretty print a module for debugging
pub fn print_module(module: &Module) -> String {
    format!("#module {} ({} functions)", module.name, module.functions.len())
}