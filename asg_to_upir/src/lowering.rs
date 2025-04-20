//! Lowering ASG to UPIR: supports System F type abs/app and ADT modeling.

use asg_core::*;
use type_checker_l2::{Type, TypeCheckResult};
use upir_core::ir::*;
use upir_core::types::*;

use std::collections::HashMap;

use crate::error::{LoweringError, Result};

pub struct LoweringContext<'a> {
    pub graph: &'a AsgGraph,
    pub upir_module: Module,
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
        }
    }
}

pub fn lower_graph_to_upir(graph: &AsgGraph) -> Result<Module> {
    let mut ctx = LoweringContext::new(graph);

    // Lowering pass: scan and convert TypeAbs, TypeApp, DataDef, DataCtor/Match
    for (node_id, node) in &graph.nodes {
        match node.type_ {
            NodeType::TypeAbs => {
                // Adds a polymorphic function (type_params passed through)
                // Stub: Just record in module.typeparam_decls for demonstration.
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
                            field_types: ctor.field_type_node_ids.iter().map(|_| TypeId(1)).collect() // Dummy type ids
                        }).collect(),
                    });
                }
            }
            NodeType::DataMatch => {
                // Insert a match operation (attributes, match_info).
                // This is a stub structure, see ir.rs OPERATION struct for match_info extension.
                // Real blocks/arms would be built here.
            }
            _ => {}
        }
    }
    Ok(ctx.upir_module)
}