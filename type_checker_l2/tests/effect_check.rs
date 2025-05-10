use asg_core::{AsgGraph, AsgNode, NodeType, EffectMeta};
use type_checker_l2::check_and_annotate_graph_v2_with_effects_check;

#[test]
fn effect_ok_all_allowed() {
    let mut graph = AsgGraph::new();
    let n1 = graph.add_node(AsgNode {
        node_id: 10,
        type_: NodeType::Unknown,
        effect_meta: Some(EffectMeta { effect_tags: vec!["IO".to_string(), "State".to_string()] }),
        ..Default::default()
    });
    assert!(check_and_annotate_graph_v2_with_effects_check(&mut graph, &["IO", "State"]).is_ok());
}

#[test]
fn effect_error_disallowed() {
    let mut graph = AsgGraph::new();
    let n2 = graph.add_node(AsgNode {
        node_id: 20,
        type_: NodeType::Unknown,
        effect_meta: Some(EffectMeta { effect_tags: vec!["IO".to_string(), "Secret".to_string()] }),
        ..Default::default()
    });
    let res = check_and_annotate_graph_v2_with_effects_check(&mut graph, &["IO"]);
    assert!(res.is_err(), "Disallowed effect should cause error");
}