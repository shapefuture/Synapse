//! Basic tests for type inference on small ASGs.

use asg_core::{AsgGraph, AsgNode, NodeType};
use type_checker_l1::check_and_annotate_graph;

#[test]
fn test_lambda_trivial_int() {
    // This is a minimal, programmatically constructed ASG representing:
    // lambda (x: Int) -> x
    let mut graph = AsgGraph::new();

    // Create a parameter variable node
    let param_node_id = graph.add_node(asg_core::AsgNode {
        node_id: 0,
        type_: NodeType::TermVariable,
        term_variable: Some(asg_core::TermVariable {
            name: "x".to_string(),
            definition_node_id: 10, // 10 will be the lambda binder
        }),
        ..Default::default()
    });

    // Create a lambda node, with binder param and body (the param node)
    let lambda_id = graph.add_node(asg_core::AsgNode {
        node_id: 10,
        type_: NodeType::TermLambda,
        term_lambda: Some(asg_core::TermLambda {
            binder_variable_node_id: param_node_id,
            body_node_id: param_node_id,
            type_annotation_id: None,
        }),
        ..Default::default()
    });

    graph.set_root(lambda_id);

    let result = check_and_annotate_graph(&mut graph);
    assert!(result.is_ok());
}