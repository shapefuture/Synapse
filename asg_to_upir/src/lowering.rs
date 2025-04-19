//! Core lowering logic: ASG → UPIR.

use asg_core::*;
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
    pub next_block: u64,
    pub type_id_map: HashMap<Type, TypeId>,
    pub next_type: u64,
}

impl<'a> LoweringContext<'a> {
    pub fn new(graph: &'a AsgGraph, type_map: &'a TypeCheckMap) -> Self {
        LoweringContext {
            graph,
            type_map,
            upir_module: Module { name: "main".to_string(), functions: vec![] },
            ir_value_map: HashMap::new(),
            next_value: 0,
            next_block: 0,
            type_id_map: HashMap::new(),
            next_type: 1,
        }
    }
    fn next_value(&mut self) -> ValueId {
        let v = self.next_value;
        self.next_value += 1;
        ValueId(v)
    }
    fn next_block(&mut self) -> BlockId {
        let b = self.next_block;
        self.next_block += 1;
        BlockId(b)
    }
    fn resolve_type(&mut self, ty: &Type) -> TypeId {
        if let Some(id) = self.type_id_map.get(ty) {
            *id
        } else {
            // In a real system, types would be registered in the UPIR module
            let new_id = TypeId(self.next_type);
            self.next_type += 1;
            self.type_id_map.insert(ty.clone(), new_id);
            new_id
        }
    }
}

/// Main entry: Lower a type-checked ASG to a UPIR Module.
/// Only supports trivial "lambda (x: Int) -> x" for now.
pub fn lower_graph_to_upir(graph: &AsgGraph, types: &TypeCheckMap) -> Result<Module> {
    let mut ctx = LoweringContext::new(graph, types);
    let root = graph.root_node_id();
    if let Some(node) = graph.nodes.get(&root) {
        match node.type_ {
            NodeType::TermLambda => {
                let fun = lower_lambda(&mut ctx, root)?;
                ctx.upir_module.functions.push(fun);
            }
            _ => return Err(LoweringError::NodeNotLowerable(root))
        }
    }
    Ok(ctx.upir_module)
}

/// Lower a simple lambda node as a UPIR function with one block that returns its param.
fn lower_lambda(ctx: &mut LoweringContext, node_id: u64) -> Result<Function> {
    let node = ctx.graph.nodes.get(&node_id).ok_or(LoweringError::NodeNotLowerable(node_id))?;
    let lambda = node.term_lambda.as_ref().ok_or(LoweringError::NodeNotLowerable(node_id))?;
    // Assume body node is a var referencing binder.
    let binder_id = lambda.binder_variable_node_id;
    let param_ty_l1 = ctx.type_map.get(&binder_id).ok_or(LoweringError::MissingType(binder_id))?;
    let param_ty = ctx.resolve_type(param_ty_l1);
    let res_ty_l1 = ctx.type_map.get(&lambda.body_node_id).ok_or(LoweringError::MissingType(lambda.body_node_id))?;
    let res_ty = ctx.resolve_type(res_ty_l1);
    let fid = format!("lambda_{}", node_id);

    // Build the body block
    let blockid = ctx.next_block();

    // Register parameter as block arg and as ValueId
    let param_val_id = ctx.next_value();
    ctx.ir_value_map.insert(binder_id, param_val_id);

    let block_arg = BlockArgument {
        value_def: ValueDef {
            id: param_val_id,
            ty: param_ty,
        },
        block_id: blockid,
        index: 0,
    };

    // Lower the body (if it's just a variable, map directly)
    let ret_val_id = if let Some(body_node) = ctx.graph.nodes.get(&lambda.body_node_id) {
        match body_node.type_ {
            NodeType::TermVariable => {
                // The parameter variable
                if let Some(termvar) = &body_node.term_variable {
                    if termvar.definition_node_id == binder_id {
                        param_val_id
                    } else {
                        // Could add lookup for variable mapping
                        param_val_id
                    }
                } else { param_val_id }
            }
            _ => param_val_id
        }
    } else { param_val_id };

    let ret_op = Operation {
        name: "func.return".to_string(),
        operands: vec![ret_val_id],
        results: vec![],
        attributes: HashMap::new(),
        regions: vec![],
    };

    let block = Block {
        id: blockid,
        arguments: vec![block_arg],
        operations: vec![ret_op],
    };

    let region = Region { blocks: vec![block] };
    let func = Function {
        name: fid,
        signature: FunctionSignature {
            arg_types: vec![param_ty],
            result_types: vec![res_ty],
        },
        regions: vec![region],
    };
    Ok(func)
}