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

    /// Map UPIR TypeId to LLVM type (currently only i32 supported)
    pub fn lower_type(&mut self, upir_type: &TypeId) -> llvm_types::BasicTypeEnum<'ctx> {
        // Demo only: treat TypeId(1) as i32, else void
        if upir_type.0 == 1 {
            self.llvm_ctx.i32_type().into()
        } else {
            self.llvm_ctx.void_type().as_basic_type_enum()
        }
    }

    /// Emit a demo function: add(i32, i32) -> i32 with single BB and return
    pub fn lower_function(&mut self, func: &Function) -> Result<()> {
        let name = &func.name;
        let ret_ty = if func.signature.result_types.len() == 1 {
            self.lower_type(&func.signature.result_types[0])
        } else {
            self.llvm_ctx.void_type().as_basic_type_enum()
        };
        let arg_types: Vec<_> = func.signature.arg_types.iter().map(|t| self.lower_type(t)).collect();
        let fn_type = ret_ty.fn_type(
            &arg_types.iter().map(|x| x as &dyn inkwell::types::BasicType).collect::<Vec<_>>(),
            false,
        );
        let llvm_f = self.llvm_module.add_function(name, fn_type, None);
        self.func_map.insert(name.clone(), llvm_f);

        // Build BBs (only single block for now)
        if let Some(region) = func.regions.get(0) {
            if let Some(block) = region.blocks.get(0) {
                let entry = self.llvm_ctx.append_basic_block(llvm_f, "entry");
                self.builder.position_at_end(entry);

                // Map arguments
                for (i, arg) in block.arguments.iter().enumerate() {
                    let val = llvm_f.get_nth_param(i as u32).unwrap();
                    self.value_map.insert(arg.value_def.id, val);
                }

                // Lower ops
                for op in &block.operations {
                    match op.name.as_str() {
                        "core.add" => {
                            // Only i32 add for now
                            let lhs = self.value_map[&op.operands[0]].into_int_value();
                            let rhs = self.value_map[&op.operands[1]].into_int_value();
                            let sum = self.builder.build_int_add(lhs, rhs, "sum");
                            self.value_map.insert(op.results[0].id, sum.as_basic_value_enum());
                        }
                        "core.constant" => {
                            // i32 const
                            if let Some(Attribute::Integer(val)) = op.attributes.get("value") {
                                let c = self.llvm_ctx.i32_type().const_int(*val as u64, true);
                                self.value_map.insert(op.results[0].id, c.as_basic_value_enum());
                            }
                        }
                        "func.return" => {
                            let retval = self.value_map.get(&op.operands[0]).cloned().unwrap();
                            self.builder.build_return(Some(&retval));
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}

/// Lowers a UPIR module to LLVM IR using inkwell.
pub fn lower_upir_to_llvm<'ctx>(
    upir: &Module,
    llvm_ctx: &'ctx Context,
) -> Result<LlvmModule<'ctx>> {
    let mut ctx = LlvmLoweringContext::new(llvm_ctx, &upir.name);

    for func in &upir.functions {
        ctx.lower_function(func)?;
    }

    Ok(ctx.llvm_module)
}