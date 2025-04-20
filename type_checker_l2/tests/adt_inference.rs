//! Test: Algebraic DataType support in type checker.

use asg_core::{AsgGraph, AsgNode, NodeType, DataDef, DataCtor, DataMatch, DataMatchArm};
use type_checker_l2::check_and_annotate_graph_v2;

#[test]
fn test_option_datatype_and_match() {
    let mut graph = AsgGraph::new();

    // data Option = Some(Int) | None
    let option_def_node = graph.add_node(AsgNode {
        node_id: 1,
        type_: NodeType::DataDef,
        data_def: Some(DataDef {
            name: "Option".to_string(),
            param_names: vec![],
            ctor_defs: vec![
                DataCtor { name: "Some".to_string(), field_type_node_ids: vec![2] },
                DataCtor { name: "None".to_string(), field_type_node_ids: vec![] },
            ],
        }),
        ..Default::default()
    });

    // Int type node for constructor argument
    let int_type_node = graph.add_node(AsgNode {
        node_id: 2,
        type_: NodeType::Unknown, // Use custom marker for type nodes
        ..Default::default()
    });

    // Build pattern match: match scrutinee with arms (stubs)
    let match_node = graph.add_node(AsgNode {
        node_id: 3,
        type_: NodeType::DataMatch,
        data_match: Some(DataMatch {
            scrutinee_node_id: option_def_node,
            arms: vec![
                DataMatchArm { ctor_name: "Some".to_string(), pattern_var_node_ids: vec![2], body_node_id: 2 },
                DataMatchArm { ctor_name: "None".to_string(), pattern_var_node_ids: vec![], body_node_id: 2 },
            ],
        }),
        ..Default::default()
    });

    graph.set_root(match_node);

    let result = check_and_annotate_graph_v2(&mut graph);
    assert!(result.is_ok());
}