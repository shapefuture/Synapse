//! Core lowering logic: ASG → UPIR.

use asg_core::AsgGraph;
use type_checker_l1::{Type, TypeCheckMap};
use upir_core::ir::*;
use upir_core::types::*;

use std::collections::HashMap;

use crate::error::{LoweringError, Result};

/// Context for lowering: holds mapping from ASG node IDs to UPIR values/types, etc.
pub struct LoweringContext<'a> {
    pub graph: &'a AsgGraph,
    pub type_map: &'a TypeCheckMap,
    pub upir_module: Module,
    pub ir_value_map: HashMap<u64, ValueId>, // ASG node id → UPIR ValueId
    pub next_value: u64,
}

impl<'a> LoweringContext<'a> {
    pub fn new(graph: &'a AsgGraph, type_map: &'a TypeCheckMap) -> Self {
        LoweringContext {
            graph,
            type_map,
            upir_module: Module { name: "main".to_string(), functions: vec![] },
            ir_value_map: HashMap::new(),
            next_value: 1_000_000,
        }
    }
}

/// Main entry: Lower a type-checked ASG to a UPIR Module.
/// Returns either the constructed UPIR Module or a lowering error.
pub fn lower_graph_to_upir(graph: &AsgGraph, types: &TypeCheckMap) -> Result<Module> {
    let mut ctx = LoweringContext::new(graph, types);
    // STUB: for now, just returns an empty module.
    // TODO: walk functions, lower lambdas, etc., producing blocks, ops, and value mapping.
    Ok(ctx.upir_module)
}