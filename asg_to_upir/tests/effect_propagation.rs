use asg_core::{AsgGraph, AsgNode, NodeType, EffectMeta, DataMatch};
use asg_to_upir::lower_graph_to_upir;

#[test]
fn propagate_effect_tag_to_upir() {
    let mut graph = AsgGraph::new();
    let data_match = graph.add_node(AsgNode {
        node_id: 10,
        type_: NodeType::DataMatch,
        effect_meta: Some(EffectMeta { effect_tags: vec!["IO".to_string(), "State".to_string()] }),
        data_match: Some(DataMatch { scrutinee_node_id: 0, arms: vec![] }),
        ..Default::default()
    });
    let upir = lower_graph_to_upir(&graph).unwrap();
    let found = upir.functions.iter().flat_map(|f| &f.regions).flat_map(|r| &r.blocks)
        .flat_map(|b| &b.operations)
        .any(|op| op.effect_tags.contains(&"IO".to_string()) && op.effect_tags.contains(&"State".to_string()));
    assert!(found, "Effect tags should propagate to UPIR operation");
}