//! Simple System F polymorphism: ∀X. λx:X. x, then apply to Int.

use asg_core::{AsgGraph, AsgNode, NodeType, TypeAbs, TypeApp};
use type_checker_l2::check_and_annotate_graph_v2;

#[test]
fn test_polymorphic_identity() {
    let mut graph = AsgGraph::new();

    // Add type parameter X = 100
    let type_param_id = 100;
    // Node for variable x: TypeVar(X)
    let var_node_id = graph.add_node(AsgNode {
        node_id: 1,
        type_: NodeType::TermVariable,
        term_variable: None,
        type_var_id: Some(type_param_id),
        ..Default::default()
    });

    // Lambda: λx:X. x
    let lambda_node_id = graph.add_node(AsgNode {
        node_id: 2,
        type_: NodeType::TermLambda,
        term_lambda: None, // Not fully populated, just schema demo.
        ..Default::default()
    });

    // Type abstraction: ∀X. λx:X. x
    let abs_node_id = graph.add_node(AsgNode {
        node_id: 3,
        type_: NodeType::TypeAbs,
        type_abs: Some(TypeAbs {
            type_param_ids: vec![type_param_id],
            body_node_id: lambda_node_id,
        }),
        ..Default::default()
    });

    // Apply type abs to Int: (∀X. λx:X. x) [Int]
    let app_node_id = graph.add_node(AsgNode {
        node_id: 4,
        type_: NodeType::TypeApp,
        type_app: Some(TypeApp {
            type_abs_node_id: abs_node_id,
            type_arg_node_ids: vec![200], // node representing Int
        }),
        ..Default::default()
    });

    graph.set_root(app_node_id);

    let result = check_and_annotate_graph_v2(&mut graph);
    assert!(result.is_ok());
}