//! Core IR structures for UPIR, covering all core dialects.

use crate::types::*;
use crate::attributes::*;
use crate::dialects::*;
use std::collections::HashMap;

/// A UPIR module contains functions.
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
    /// Op name: e.g. "core.add", "mem.load"
    pub name: String,
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

/// Top-level dialect operation kind enum (for matching in switches)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpKind {
    // Dialect::opname
    BuiltinModule,
    BuiltinFunc,
    CoreConstant,
    CoreAdd,
    CoreSub,
    CoreMul,
    CoreDivS,
    CoreDivU,
    CoreRemS,
    CoreRemU,
    CoreAnd,
    CoreOr,
    CoreXor,
    CoreCmp, // attr: predicate (EQ, NE, etc)
    MemAlloc,
    MemLoad,
    MemStore,
    CfBr,
    CfCondBr,
    FuncCall,
    FuncReturn,
    // ...add extension ops as needed...
    Custom(String), // fallback
}

impl OpKind {
    pub fn from_str(s: &str) -> Self {
        match s {
            "builtin.module" => OpKind::BuiltinModule,
            "builtin.func" => OpKind::BuiltinFunc,
            "core.constant" => OpKind::CoreConstant,
            "core.add" => OpKind::CoreAdd,
            "core.sub" => OpKind::CoreSub,
            "core.mul" => OpKind::CoreMul,
            "core.div_s" => OpKind::CoreDivS,
            "core.div_u" => OpKind::CoreDivU,
            "core.rem_s" => OpKind::CoreRemS,
            "core.rem_u" => OpKind::CoreRemU,
            "core.and" => OpKind::CoreAnd,
            "core.or" => OpKind::CoreOr,
            "core.xor" => OpKind::CoreXor,
            "core.cmp" => OpKind::CoreCmp,
            "mem.alloc" => OpKind::MemAlloc,
            "mem.load" => OpKind::MemLoad,
            "mem.store" => OpKind::MemStore,
            "cf.br" => OpKind::CfBr,
            "cf.cond_br" => OpKind::CfCondBr,
            "func.call" => OpKind::FuncCall,
            "func.return" => OpKind::FuncReturn,
            _ => OpKind::Custom(s.to_string())
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            OpKind::BuiltinModule => "builtin.module",
            OpKind::BuiltinFunc => "builtin.func",
            OpKind::CoreConstant => "core.constant",
            OpKind::CoreAdd => "core.add",
            OpKind::CoreSub => "core.sub",
            OpKind::CoreMul => "core.mul",
            OpKind::CoreDivS => "core.div_s",
            OpKind::CoreDivU => "core.div_u",
            OpKind::CoreRemS => "core.rem_s",
            OpKind::CoreRemU => "core.rem_u",
            OpKind::CoreAnd => "core.and",
            OpKind::CoreOr => "core.or",
            OpKind::CoreXor => "core.xor",
            OpKind::CoreCmp => "core.cmp",
            OpKind::MemAlloc => "mem.alloc",
            OpKind::MemLoad => "mem.load",
            OpKind::MemStore => "mem.store",
            OpKind::CfBr => "cf.br",
            OpKind::CfCondBr => "cf.cond_br",
            OpKind::FuncCall => "func.call",
            OpKind::FuncReturn => "func.return",
            OpKind::Custom(s) => s.as_str(),
        }
    }
}

/// Builder pattern for safe construction (example for Module/Function)
pub struct IRBuilder {
    pub id_counter: u64,
}

impl IRBuilder {
    pub fn new() -> Self { Self { id_counter: 10_000 } }

    /// Create a module skeleton
    pub fn module(&mut self, name: &str) -> Module {
        Module {
            name: name.to_string(),
            functions: vec![],
        }
    }
    // Add functions/blocks/etc. helpers as needed.
}

/// Pretty print a module in MLIR-inspired syntax.
pub fn print_module(module: &Module) -> String {
    let mut s = String::new();
    s.push_str(&format!("module @{} {{\n", module.name));
    for function in &module.functions {
        s.push_str(&print_function(function));
    }
    s.push_str("}\n");
    s
}

fn print_function(function: &Function) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "  func @{}(",
        function.name
    ));
    for (i, t) in function.signature.arg_types.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push_str(&format!("%arg{}: type{}", i, t.0));
    }
    s.push_str(") -> (");
    for (i, t) in function.signature.result_types.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push_str(&format!("type{}", t.0));
    }
    s.push_str(") {\n");
    for region in &function.regions {
        for block in &region.blocks {
            s.push_str(&print_block(block));
        }
    }
    s.push_str("  }\n");
    s
}

fn print_block(block: &Block) -> String {
    let mut s = String::new();
    s.push_str(&format!("    ^bb{}(", block.id.0));
    for (i, arg) in block.arguments.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push_str(&format!("%v{}: type{}", i, arg.value_def.ty.0));
    }
    s.push_str("):\n");
    for op in &block.operations {
        s.push_str(&print_operation(op));
    }
    s
}

fn print_operation(op: &Operation) -> String {
    let mut s = String::new();
    for (i, result) in op.results.iter().enumerate() {
        s.push_str(&format!("%r{}: type{} = ", i, result.ty.0));
    }
    s.push_str(&format!("{}(", op.name));
    for (i, operand) in op.operands.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push_str(&format!("%v{}", operand.0));
    }
    if !op.attributes.is_empty() {
        let attrs: Vec<String> = op.attributes.iter().map(|(k, v)| format!("{}={:?}", k, v)).collect();
        s.push_str(&format!("; {}", attrs.join(", ")));
    }
    s.push_str(")\n");
    s
}