use asg_core::{AsgGraph, AsgNode, NodeType};
use type_checker_l1::check_and_annotate_graph;
use asg_to_upir::lower_graph_to_upir;

// Only a stub test for now—expand as lowering is completed.
#[test]
fn trivial_empty_graph_roundtrip() {
    let mut graph = AsgGraph::new();
    let type_map = check_and_annotate_graph(&mut graph).expect("Type check should succeed");
    let result = lower_graph_to_upir(&graph, &type_map);
    assert!(result.is_ok());
}