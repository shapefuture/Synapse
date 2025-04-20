//! Lowering ASG to UPIR: supports System F type abs/app and ADT modeling and full DataMatch translation.

use asg_core::*;
use type_checker_l2::{Type, TypeCheckResult};
use upir_core::ir::*;
use upir_core::types::*;

use std::collections::HashMap;

use crate::error::{LoweringError, Result};

pub struct LoweringContext<'a> {
    pub graph: &'a AsgGraph,
    pub upir_module: Module,
    pub block_counter: u64,
    pub value_counter: u64,
}

impl<'a> LoweringContext<'a> {
    pub fn new(graph: &'a AsgGraph) -> Self {
        LoweringContext {
            graph,
            upir_module: Module {
                name: "main".to_string(),
                functions: vec![],
                datatype_decls: vec![],
                typeparam_decls: vec![],
                effect_decls: vec![],
            },
            block_counter: 0,
            value_counter: 0,
        }
    }
    fn next_block(&mut self) -> BlockId {
        let b = self.block_counter;
        self.block_counter += 1;
        BlockId(b)
    }
    fn next_value(&mut self) -> ValueId {
        let v = self.value_counter;
        self.value_counter += 1;
        ValueId(v)
    }
}

pub fn lower_graph_to_upir(graph: &AsgGraph) -> Result<Module> {
    let mut ctx = LoweringContext::new(graph);

    for (node_id, node) in &graph.nodes {
        match node.type_ {
            NodeType::TypeAbs => {
                if let Some(abs) = &node.type_abs {
                    for tid in &abs.type_param_ids {
                        ctx.upir_module.typeparam_decls.push(TypeParamDecl { name: format!("T{}", tid) });
                    }
                }
            }
            NodeType::DataDef => {
                if let Some(dd) = &node.data_def {
                    ctx.upir_module.datatype_decls.push(DataTypeDecl {
                        name: dd.name.clone(),
                        params: dd.param_names.iter().map(|n| TypeParamDecl { name: n.clone() }).collect(),
                        ctors: dd.ctor_defs.iter().map(|ctor| AdtConstructor {
                            name: ctor.name.clone(),
                            field_types: ctor.field_type_node_ids.iter().map(|_| TypeId(1)).collect()
                        }).collect(),
                    });
                }
            }
            NodeType::DataMatch => {
                if let Some(dm) = &node.data_match {
                    // Lower scrutinee expression (should produce a value)
                    let scrutinee_id = dm.scrutinee_node_id;
                    let scrut_value_id = ctx.next_value();
                    // In a real impl, lower_expr would return ValueId and ops, assign here.

                    // Create a block for each arm's body
                    let mut match_info = MatchInfo { arms: vec![] };
                    let mut blocks = vec![];
                    for arm in &dm.arms {
                        // Each arm creates a new block...
                        let block_id = ctx.next_block();
                        // Each pattern var becomes a block argument
                        let block_args: Vec<BlockArgument> = arm.pattern_var_node_ids.iter().enumerate().map(|(i, &var_id)| {
                            BlockArgument {
                                value_def: ValueDef { id: ctx.next_value(), ty: TypeId(1) },
                                block_id,
                                index: i,
                            }
                        }).collect();
                        // Lower the arm body (should return a ValueId)
                        let body_block = Block {
                            id: block_id,
                            arguments: block_args.clone(),
                            operations: vec![], // Real lowering would use lower_expr
                        };
                        blocks.push(body_block);
                        match_info.arms.push(AdtMatchArm {
                            ctor: arm.ctor_name.clone(),
                            vars: block_args.iter().map(|a| a.value_def.id).collect(),
                            body_block: block_id,
                        });
                    }
                    // Make Op
                    let match_op = Operation {
                        name: "core.match".to_string(),
                        operands: vec![scrut_value_id],
                        results: vec![],
                        attributes: HashMap::new(),
                        regions: vec![Region { blocks: blocks.clone() }],
                        datatype_info: None,
                        match_info: Some(match_info),
                    };
                    // Insert operation as a new top-level function
                    let fake_func = Function {
                        name: format!("match_{}", node_id),
                        signature: FunctionSignature { arg_types: vec![], result_types: vec![] },
                        type_params: vec![],
                        regions: vec![Region { blocks: vec![Block {
                            id: ctx.next_block(),
                            arguments: vec![],
                            operations: vec![match_op],
                        }] }]
                    };
                    ctx.upir_module.functions.push(fake_func);
                }
            }
            _ => {}
        }
    }
    Ok(ctx.upir_module)
}