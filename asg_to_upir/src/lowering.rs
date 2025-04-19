//! Core lowering logic: ASG → UPIR.

use asg_core::*;
use type_checker_l1::{Type, TypeCheckMap};
use upir_core::ir::*;
use upir_core::types::*;

use std::collections::HashMap;

use crate::error::{LoweringError, Result};

/// Context for lowering: tracks mapping from ASG node IDs to UPIR SSA values and types.
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

/// Main entry: fully lower a type-checked ASG to a UPIR module.
pub fn lower_graph_to_upir(graph: &AsgGraph, types: &TypeCheckMap) -> Result<Module> {
    let mut ctx = LoweringContext::new(graph, types);
    let root = graph.root_node_id();
    if let Some(node) = graph.nodes.get(&root) {
        match node.type_ {
            NodeType::TermLambda => {
                let fun = lower_lambda(&mut ctx, root)?;
                ctx.upir_module.functions.push(fun);
            }
            _ => return Err(LoweringError::NodeNotLowerable(root)),
        }
    }
    Ok(ctx.upir_module)
}

/// Recursively lower a lambda as a UPIR function, supporting body expressions.
fn lower_lambda(ctx: &mut LoweringContext, node_id: u64) -> Result<Function> {
    let node = ctx.graph.nodes.get(&node_id).ok_or(LoweringError::NodeNotLowerable(node_id))?;
    let lambda = node.term_lambda.as_ref().ok_or(LoweringError::NodeNotLowerable(node_id))?;
    let binder_id = lambda.binder_variable_node_id;
    let param_ty_hm = ctx.type_map.get(&binder_id).ok_or(LoweringError::MissingType(binder_id))?;
    let param_ty = ctx.resolve_type(param_ty_hm);
    let res_ty_hm = ctx.type_map.get(&lambda.body_node_id).ok_or(LoweringError::MissingType(lambda.body_node_id))?;
    let res_ty = ctx.resolve_type(res_ty_hm);
    let fid = format!("lambda_{}", node_id);

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

    // Lower body recursively: may be application, op, variable, literal.
    let (ret_val_id, mut ops) = lower_expr(ctx, lambda.body_node_id, blockid)?;

    let ret_op = Operation {
        name: "func.return".to_string(),
        operands: vec![ret_val_id],
        results: vec![],
        attributes: HashMap::new(),
        regions: vec![],
    };
    ops.push(ret_op);

    let block = Block {
        id: blockid,
        arguments: vec![block_arg],
        operations: ops,
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

/// Recursively lowers an expression and returns its resulting ValueId and list of ops.
fn lower_expr(ctx: &mut LoweringContext, node_id: u64, block: BlockId) -> Result<(ValueId, Vec<Operation>)> {
    let node = ctx.graph.nodes.get(&node_id).ok_or(LoweringError::NodeNotLowerable(node_id))?;
    let ty = ctx.type_map.get(&node_id).ok_or(LoweringError::MissingType(node_id))?;
    let typid = ctx.resolve_type(ty);
    match node.type_ {
        NodeType::TermVariable => {
            // Map to existing SSA value if available.
            if let Some(val_id) = ctx.ir_value_map.get(&node_id) {
                Ok((*val_id, vec![]))
            } else if let Some(tv) = &node.term_variable {
                // Try mapping to definition binder.
                if let Some(val_id) = ctx.ir_value_map.get(&tv.definition_node_id) {
                    ctx.ir_value_map.insert(node_id, *val_id);
                    Ok((*val_id, vec![]))
                } else {
                    Err(LoweringError::NodeNotLowerable(node_id))
                }
            } else {
                Err(LoweringError::NodeNotLowerable(node_id))
            }
        },
        NodeType::LiteralInt => {
            let val_id = ctx.next_value();
            ctx.ir_value_map.insert(node_id, val_id);
            let mut attrs = HashMap::new();
            attrs.insert("value".to_string(), Attribute::Integer(node.literal_int.unwrap_or(0)));
            let op = Operation {
                name: "core.constant".to_string(),
                operands: vec![],
                results: vec![ValueDef { id: val_id, ty: typid }],
                attributes: attrs,
                regions: vec![],
            };
            Ok((val_id, vec![op]))
        },
        NodeType::LiteralBool => {
            let val_id = ctx.next_value();
            ctx.ir_value_map.insert(node_id, val_id);
            let mut attrs = HashMap::new();
            attrs.insert("value".to_string(), Attribute::Bool(node.literal_bool.unwrap_or(false)));
            let op = Operation {
                name: "core.constant".to_string(),
                operands: vec![],
                results: vec![ValueDef { id: val_id, ty: typid }],
                attributes: attrs,
                regions: vec![],
            };
            Ok((val_id, vec![op]))
        },
        NodeType::TermApplication => {
            // Lower function and argument.
            if let Some(app) = &node.term_application {
                let (fn_val, mut fn_ops) = lower_expr(ctx, app.function_node_id, block)?;
                let (arg_val, mut arg_ops) = lower_expr(ctx, app.argument_node_id, block)?;
                let val_id = ctx.next_value();
                ctx.ir_value_map.insert(node_id, val_id);
                let call_op = Operation {
                    name: "func.call".to_string(),
                    operands: vec![fn_val, arg_val],
                    results: vec![ValueDef { id: val_id, ty: typid }],
                    attributes: HashMap::new(),
                    regions: vec![],
                };
                fn_ops.append(&mut arg_ops);
                fn_ops.push(call_op);
                Ok((val_id, fn_ops))
            } else {
                Err(LoweringError::Unimplemented("term application missing".to_string()))
            }
        },
        NodeType::PrimitiveOp => {
            if let Some(op) = &node.primitive_op {
                // Lower all argument nodes
                let mut result_ops = vec![];
                let mut arg_vals = vec![];
                for &arg_node in &op.argument_node_ids {
                    let (arg_val, mut ops) = lower_expr(ctx, arg_node, block)?;
                    result_ops.append(&mut ops);
                    arg_vals.push(arg_val);
                }
                let val_id = ctx.next_value();
                ctx.ir_value_map.insert(node_id, val_id);
                let ir_op = Operation {
                    name: format!("core.{}", op.op_name),
                    operands: arg_vals,
                    results: vec![ValueDef { id: val_id, ty: typid }],
                    attributes: HashMap::new(),
                    regions: vec![],
                };
                result_ops.push(ir_op);
                Ok((val_id, result_ops))
            } else {
                Err(LoweringError::Unimplemented("primitive op".to_string()))
            }
        }
        NodeType::TermRef => {
            if let Some(term_ref) = &node.term_ref {
                let (val, mut ops) = lower_expr(ctx, term_ref.init_value_node_id, block)?;
                let ref_id = ctx.next_value();
                let alloc_op = Operation {
                    name: "mem.alloc".to_string(),
                    operands: vec![], // type could go as attribute
                    results: vec![ValueDef { id: ref_id, ty: typid }],
                    attributes: HashMap::new(),
                    regions: vec![],
                };
                let store_op = Operation {
                    name: "mem.store".to_string(),
                    operands: vec![ref_id, val],
                    results: vec![],
                    attributes: HashMap::new(),
                    regions: vec![],
                };
                ctx.ir_value_map.insert(node_id, ref_id);
                ops.push(alloc_op);
                ops.push(store_op);
                Ok((ref_id, ops))
            } else {
                Err(LoweringError::Unimplemented("term ref".to_string()))
            }
        }
        NodeType::TermDeref => {
            if let Some(deref) = &node.term_deref {
                let (ptr_val, mut ops) = lower_expr(ctx, deref.ref_node_id, block)?;
                let result_id = ctx.next_value();
                let load = Operation {
                    name: "mem.load".to_string(),
                    operands: vec![ptr_val],
                    results: vec![ValueDef { id: result_id, ty: typid }],
                    attributes: HashMap::new(),
                    regions: vec![],
                };
                ctx.ir_value_map.insert(node_id, result_id);
                ops.push(load);
                Ok((result_id, ops))
            } else {
                Err(LoweringError::Unimplemented("term deref".to_string()))
            }
        }
        NodeType::TermAssign => {
            if let Some(assign) = &node.term_assign {
                let (ref_val, mut ref_ops) = lower_expr(ctx, assign.ref_node_id, block)?;
                let (value_val, mut val_ops) = lower_expr(ctx, assign.value_node_id, block)?;
                let assign_op = Operation {
                    name: "mem.store".to_string(),
                    operands: vec![ref_val, value_val],
                    results: vec![],
                    attributes: HashMap::new(),
                    regions: vec![],
                };
                ref_ops.append(&mut val_ops);
                ref_ops.push(assign_op);
                // Assignment returns bool/unit; synthesize a dummy constant as result
                let result_id = ctx.next_value();
                let dummy_true = Operation {
                    name: "core.constant".to_string(),
                    operands: vec![],
                    results: vec![ValueDef { id: result_id, ty: typid }],
                    attributes: {
                        let mut attrs = HashMap::new();
                        attrs.insert("value".to_string(), Attribute::Bool(true));
                        attrs
                    },
                    regions: vec![],
                };
                ctx.ir_value_map.insert(node_id, result_id);
                ref_ops.push(dummy_true);
                Ok((result_id, ref_ops))
            } else {
                Err(LoweringError::Unimplemented("term assign".to_string()))
            }
        }
        _ => Err(LoweringError::NodeNotLowerable(node_id)),
    }
}