use asg_core::{AsgGraph, AsgNode, NodeType, TypeAbs, DataDef, DataCtor};
use asg_to_upir::lower_graph_to_upir;
use upir_core::ir::print_module;

#[test]
fn lower_poly_and_adt() {
    let mut graph = AsgGraph::new();
    // Build a type abstraction node
    let abs_node = graph.add_node(AsgNode {
        node_id: 40,
        type_: NodeType::TypeAbs,
        type_abs: Some(TypeAbs {
            type_param_ids: vec![1],
            body_node_id: 41,
        }),
        ..Default::default()
    });
    // Build ADT node: data Either = Left(Int) | Right(Bool)
    let adt_node = graph.add_node(AsgNode {
        node_id: 50,
        type_: NodeType::DataDef,
        data_def: Some(DataDef {
            name: "Either".to_string(),
            param_names: vec![],
            ctor_defs: vec![
                DataCtor { name: "Left".to_string(), field_type_node_ids: vec![60] },
                DataCtor { name: "Right".to_string(), field_type_node_ids: vec![61] },
            ]
        }),
        ..Default::default()
    });
    // Lower and check IR structure
    let result = lower_graph_to_upir(&graph).expect("Should lower");
    let src = print_module(&result);
    println!("{}", src);
    assert!(src.contains("Either"));
    assert!(src.contains("T1"));
}