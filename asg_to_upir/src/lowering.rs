//! Lowering: propagate effect tags from ASG into UPIR.

use asg_core::*;
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
                effect_tags: vec![], // propagate module-level if needed
            },
            block_counter: 0,
            value_counter: 0,
        }
    }
    // ... rest as before ...
}

pub fn lower_graph_to_upir(graph: &AsgGraph) -> Result<Module> {
    let mut ctx = LoweringContext::new(graph);

    for (node_id, node) in &graph.nodes {
        let node_effects = node.effect_meta.as_ref().map(|meta| meta.effect_tags.clone()).unwrap_or_default();
        match node.type_ {
            // ... previous matches ...
            NodeType::DataMatch => {
                if let Some(dm) = &node.data_match {
                    // As before, plus effect tag propagation
                    let match_op = Operation {
                        name: "core.match".to_string(),
                        operands: vec![],
                        results: vec![],
                        attributes: HashMap::new(),
                        regions: vec![],
                        datatype_info: None,
                        match_info: None,
                        effect_tags: node_effects,
                    };
                    // Insert as a fake function just as before
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