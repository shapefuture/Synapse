//! Core UPIR → LLVM lowering logic.

use upir_core::ir::*;
use upir_core::types::*;
use inkwell::context::Context;
use inkwell::module::Module as LlvmModule;
use inkwell::values::*;
use inkwell::types as llvm_types;
use std::collections::HashMap;

use crate::error::{Result, LlvmLoweringError};

pub struct LlvmLoweringContext<'ctx> {
    pub llvm_ctx: &'ctx Context,
    pub llvm_module: LlvmModule<'ctx>,
    pub builder: inkwell::builder::Builder<'ctx>,
    pub value_map: HashMap<ValueId, BasicValueEnum<'ctx>>,
    pub type_map: HashMap<TypeId, llvm_types::BasicTypeEnum<'ctx>>,
    pub func_map: HashMap<String, inkwell::values::FunctionValue<'ctx>>,
}

impl<'ctx> LlvmLoweringContext<'ctx> {
    pub fn new(llvm_ctx: &'ctx Context, name: &str) -> Self {
        let llvm_module = llvm_ctx.create_module(name);
        let builder = llvm_ctx.create_builder();
        Self {
            llvm_ctx,
            llvm_module,
            builder,
            value_map: HashMap::new(),
            type_map: HashMap::new(),
            func_map: HashMap::new(),
        }
    }
}

/// Lowers a UPIR module to LLVM using inkwell.
/// Returns the generated LLVM module.
pub fn lower_upir_to_llvm<'ctx>(
    upir: &Module,
    llvm_ctx: &'ctx Context,
) -> Result<LlvmModule<'ctx>> {
    let mut ctx = LlvmLoweringContext::new(llvm_ctx, &upir.name);
    // STUB: No actual lowering yet.
    Ok(ctx.llvm_module)
}