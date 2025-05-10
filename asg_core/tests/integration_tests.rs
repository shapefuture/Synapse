use asg_core::{
    AsgGraph, LiteralInt, LiteralBool, NodeType, TermLambda, TermVariable, 
    TermApplication, hash_node, save_asg_binary, load_asg_binary
};
use std::path::PathBuf;
use std::fs;

#[test]
fn test_node_creation_and_retrieval() {
    let mut graph = AsgGraph::new();
    
    // Create a simple lambda: (x: Int) => x
    let x_var_id = graph.add_node(
        NodeType::TERM_VARIABLE,
        TermVariable {
            name: "x".to_string(),
            definition_node_id: 0, // Will be updated later
        },
    );
    
    let body_id = graph.add_node(
        NodeType::TERM_VARIABLE,
        TermVariable {
            name: "x".to_string(),
            definition_node_id: x_var_id,
        },
    );
    
    let lambda_id = graph.add_node(
        NodeType::TERM_LAMBDA,
        TermLambda {
            binder_variable_node_id: x_var_id,
            body_node_id: body_id,
            type_annotation_id: 0, // No type annotation yet
        },
    );
    
    // Update the definition_node_id for the binder
    if let Some(node) = graph.get_node_mut(x_var_id) {
        if let Some(asg_core::generated::asg_node::Content::TermVariable(var)) = &mut node.content {
            let var = var.clone();
            let mut updated_var = var;
            updated_var.definition_node_id = lambda_id;
            node.content = Some(asg_core::generated::asg_node::Content::TermVariable(updated_var));
        }
    }
    
    // Set the lambda as the root
    graph.set_root(lambda_id).unwrap();
    
    // Verify retrieval
    assert_eq!(graph.root_id(), Some(lambda_id));
    assert!(graph.contains_node(x_var_id));
    assert!(graph.contains_node(body_id));
    assert!(graph.contains_node(lambda_id));
    
    // Verify lambda structure
    let lambda = graph.get_lambda(lambda_id).unwrap();
    assert_eq!(lambda.binder_variable_node_id, x_var_id);
    assert_eq!(lambda.body_node_id, body_id);
    
    // Verify variable reference
    let body_var = graph.get_variable(body_id).unwrap();
    assert_eq!(body_var.name, "x");
    assert_eq!(body_var.definition_node_id, x_var_id);
}

#[test]
fn test_application() {
    let mut graph = AsgGraph::new();
    
    // Create f = (x: Int) => x + 1
    let x_var_id = graph.add_node(
        NodeType::TERM_VARIABLE,
        TermVariable {
            name: "x".to_string(),
            definition_node_id: 0,
        },
    );
    
    let one_id = graph.add_node(
        NodeType::LITERAL_INT,
        LiteralInt {
            value: 1,
        },
    );
    
    let add_op_id = graph.add_node(
        NodeType::PRIMITIVE_OP,
        asg_core::PrimitiveOp {
            op_name: "add".to_string(),
            argument_node_ids: vec![x_var_id, one_id],
        },
    );
    
    let lambda_id = graph.add_node(
        NodeType::TERM_LAMBDA,
        TermLambda {
            binder_variable_node_id: x_var_id,
            body_node_id: add_op_id,
            type_annotation_id: 0,
        },
    );
    
    // Update x variable's definition
    if let Some(node) = graph.get_node_mut(x_var_id) {
        if let Some(asg_core::generated::asg_node::Content::TermVariable(var)) = &mut node.content {
            let var = var.clone();
            let mut updated_var = var;
            updated_var.definition_node_id = lambda_id;
            node.content = Some(asg_core::generated::asg_node::Content::TermVariable(updated_var));
        }
    }
    
    // Create the argument: 42
    let arg_id = graph.add_node(
        NodeType::LITERAL_INT,
        LiteralInt {
            value: 42,
        },
    );
    
    // Create application: f(42)
    let app_id = graph.add_node(
        NodeType::TERM_APPLICATION,
        TermApplication {
            function_node_id: lambda_id,
            argument_node_id: arg_id,
        },
    );
    
    graph.set_root(app_id).unwrap();
    
    // Verify application structure
    let app = graph.get_application(app_id).unwrap();
    assert_eq!(app.function_node_id, lambda_id);
    assert_eq!(app.argument_node_id, arg_id);
}

#[test]
fn test_serialization_roundtrip() {
    let mut graph = AsgGraph::new();
    
    // Create a simple Boolean literal
    let bool_id = graph.add_node(
        NodeType::LITERAL_BOOL,
        LiteralBool {
            value: true,
        },
    );
    
    graph.set_root(bool_id).unwrap();
    
    // Create a temporary file for testing
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("test_graph.asg");
    
    // Save the graph
    save_asg_binary(&graph, &file_path).unwrap();
    
    // Load the graph back
    let loaded_graph = load_asg_binary(&file_path).unwrap();
    
    // Verify the loaded graph
    assert_eq!(loaded_graph.root_id(), Some(bool_id));
    assert!(loaded_graph.contains_node(bool_id));
    
    if let Some(node) = loaded_graph.get_node(bool_id) {
        if let Some(asg_core::generated::asg_node::Content::LiteralBool(lit)) = &node.content {
            assert_eq!(lit.value, true);
        } else {
            panic!("Expected LiteralBool content");
        }
    } else {
        panic!("Node not found in loaded graph");
    }
    
    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_hashing() {
    let mut graph1 = AsgGraph::new();
    let mut graph2 = AsgGraph::new();
    
    // Create identical nodes in both graphs
    let node1_id = graph1.add_node(
        NodeType::LITERAL_INT,
        LiteralInt {
            value: 42,
        },
    );
    
    let node2_id = graph2.add_node(
        NodeType::LITERAL_INT,
        LiteralInt {
            value: 42,
        },
    );
    
    // Get the nodes and compute their hashes
    let node1 = graph1.get_node(node1_id).unwrap();
    let node2 = graph2.get_node(node2_id).unwrap();
    
    let hash1 = hash_node(node1);
    let hash2 = hash_node(node2);
    
    // Hashes should be equal because the content is the same
    assert_eq!(hash1, hash2);
    
    // Modify one node to have a different value
    let node3_id = graph2.add_node(
        NodeType::LITERAL_INT,
        LiteralInt {
            value: 43, // Different value
        },
    );
    
    let node3 = graph2.get_node(node3_id).unwrap();
    let hash3 = hash_node(node3);
    
    // Hashes should be different
    assert_ne!(hash1, hash3);
}