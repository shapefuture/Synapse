//! Core UPIR → LLVM lowering logic with DataMatch/ADT support stub.

use upir_core::ir::*;
use upir_core::types::*;
use inkwell::context::Context;
use inkwell::module::Module as LlvmModule;
use inkwell::values::*;
use inkwell::types as llvm_types;
use inkwell::IntPredicate;
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

    pub fn lower_match_stub(&mut self, op: &Operation) {
        // Stub: show in IR as a comment, since full sum-type layout/codegen not yet implemented.
        if let Some(match_info) = &op.match_info {
            let comment = format!(
                "; UPIR core.match ({} arms: {:?})",
                match_info.arms.len(),
                match_info.arms.iter().map(|a| &a.ctor).collect::<Vec<_>>()
            );
            self.llvm_module.add_global_string(&comment, "synapse.match");
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

    for fun in &upir.functions {
        // Map argument and return types (builtin only)
        let mut llvm_arg_types = vec![];
        for tid in &fun.signature.arg_types {
            let kind = TypeKind::Builtin(BuiltinType::I32); // For now: assume all are i32 for test
            if let Some(ty) = ctx.resolve_type(&kind) {
                llvm_arg_types.push(ty.into());
            }
        }
        let ret_type = if let Some(tid) = fun.signature.result_types.get(0) {
            let kind = TypeKind::Builtin(BuiltinType::I32); // For now: only i32
            ctx.resolve_type(&kind)
        } else {
            None
        };

        let fn_type = match ret_type {
            Some(ret_ty) => ret_ty.fn_type(&llvm_arg_types, false),
            None => ctx.llvm_ctx.void_type().fn_type(&llvm_arg_types, false),
        };
        let function = ctx
            .llvm_module
            .add_function(&fun.name, fn_type, None);
        ctx.func_map.insert(fun.name.clone(), function);

        // Lower regions/blocks/ops
        for region in &fun.regions {
            for block in &region.blocks {
                let bb = ctx
                    .llvm_ctx
                    .append_basic_block(function, &format!("bb{}", block.id.0));
                ctx.builder.position_at_end(bb);

                // Map block args to parameters
                for (i, arg) in block.arguments.iter().enumerate() {
                    if let Some(param) = function.get_nth_param(i as u32) {
                        param.set_name(&format!("arg{}", i));
                        ctx.value_map.insert(arg.value_def.id, param.into());
                    }
                }

                // Lower each operation (core.add, func.return, core.match, etc)
                for op in &block.operations {
                    match op.name.as_str() {
                        "core.constant" => {
                            if let Some(Attribute::Integer(val)) = op.attributes.get("value") {
                                let const_val = ctx.llvm_ctx.i32_type().const_int(*val as u64, true).into();
                                let result_val = const_val;
                                if let Some(result) = op.results.get(0) {
                                    ctx.value_map.insert(result.id, result_val);
                                }
                            }
                        }
                        "core.add" => {
                            let lhs = ctx.value_map[&op.operands[0]].into_int_value();
                            let rhs = ctx.value_map[&op.operands[1]].into_int_value();
                            let val = ctx.builder.build_int_add(lhs, rhs, "addtmp").into();
                            if let Some(result) = op.results.get(0) {
                                ctx.value_map.insert(result.id, val);
                            }
                        }
                        "func.return" => {
                            if let Some(arg) = op.operands.get(0) {
                                let ret_val = ctx.value_map[arg];
                                ctx.builder.build_return(Some(&ret_val));
                            } else {
                                ctx.builder.build_return(None);
                            }
                        }
                        "core.match" => {
                            ctx.lower_match_stub(op);
                            // Real lowering would be here in the future.
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(ctx.llvm_module)
}