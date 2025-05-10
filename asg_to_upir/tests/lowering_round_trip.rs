use asg_core::{AsgGraph, AsgNode, NodeType, TermVariable, TermLambda, PrimitiveOp};
use type_checker_l1::check_and_annotate_graph;
use asg_to_upir::lower_graph_to_upir;
use upir_core::ir::print_module;

#[test]
fn trivial_lambda_lowering() {
    // ASG: lambda (x: Int) -> x + 42
    let mut graph = AsgGraph::new();

    // Node for 42
    let lit_id = graph.add_node(AsgNode {
        node_id: 1,
        type_: NodeType::LiteralInt,
        literal_int: Some(42),
        ..Default::default()
    });

    // Parameter variable node
    let param_node_id = graph.add_node(AsgNode {
        node_id: 2,
        type_: NodeType::TermVariable,
        term_variable: Some(TermVariable {
            name: "x".to_string(),
            definition_node_id: 10, // Binder
        }),
        ..Default::default()
    });

    // x + 42 primitive op node
    let add_node_id = graph.add_node(AsgNode {
        node_id: 3,
        type_: NodeType::PrimitiveOp,
        primitive_op: Some(PrimitiveOp {
            op_name: "+".to_string(),
            argument_node_ids: vec![param_node_id, lit_id],
        }),
        ..Default::default()
    });

    // Lambda
    let lambda_id = graph.add_node(AsgNode {
        node_id: 10,
        type_: NodeType::TermLambda,
        term_lambda: Some(TermLambda {
            binder_variable_node_id: param_node_id,
            body_node_id: add_node_id,
            type_annotation_id: None,
        }),
        ..Default::default()
    });

    graph.set_root(lambda_id);

    let type_map = check_and_annotate_graph(&mut graph).expect("Type check should succeed");
    let result = lower_graph_to_upir(&graph, &type_map).expect("Lowering should succeed");
    let pretty = print_module(&result);
    println!("{}", pretty);

    assert!(pretty.contains("core.add"));
    assert!(pretty.contains("core.constant"));
    assert!(pretty.contains("func.return"));
}