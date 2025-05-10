use asg_core::{AsgGraph, NodeType};
use parser_core::{parse_source, error::Result};

#[test]
fn test_parse_lambda() -> Result<()> {
    // (x: Int) => x + 1
    let source = "(x: Int) => x + 1";
    let graph = parse_source(source, None)?;
    
    // The root should be a lambda
    let root_id = graph.root_id().unwrap();
    let root_node = graph.get_node(root_id).unwrap();
    assert_eq!(root_node.r#type, NodeType::TERM_LAMBDA as i32);
    
    // Get the lambda
    let lambda = graph.get_lambda(root_id)?;
    
    // Check binder
    let binder_id = lambda.binder_variable_node_id;
    let binder_node = graph.get_node(binder_id).unwrap();
    assert_eq!(binder_node.r#type, NodeType::TERM_VARIABLE as i32);
    let binder = graph.get_variable(binder_id)?;
    assert_eq!(binder.name, "x");
    
    // Check body (should be a primitive operation)
    let body_id = lambda.body_node_id;
    let body_node = graph.get_node(body_id).unwrap();
    assert_eq!(body_node.r#type, NodeType::PRIMITIVE_OP as i32);
    
    Ok(())
}

#[test]
fn test_parse_application() -> Result<()> {
    // ((x: Int) => x + 1)(42)
    let source = "((x: Int) => x + 1)(42)";
    let graph = parse_source(source, None)?;
    
    // The root should be an application
    let root_id = graph.root_id().unwrap();
    let root_node = graph.get_node(root_id).unwrap();
    assert_eq!(root_node.r#type, NodeType::TERM_APPLICATION as i32);
    
    // Get the application
    let app = graph.get_application(root_id)?;
    
    // Check function (should be a lambda)
    let func_id = app.function_node_id;
    let func_node = graph.get_node(func_id).unwrap();
    assert_eq!(func_node.r#type, NodeType::TERM_LAMBDA as i32);
    
    // Check argument (should be an int literal)
    let arg_id = app.argument_node_id;
    let arg_node = graph.get_node(arg_id).unwrap();
    assert_eq!(arg_node.r#type, NodeType::LITERAL_INT as i32);
    
    Ok(())
}

#[test]
fn test_parse_reference_operations() -> Result<()> {
    // let r = ref 42; !r := 10
    let source = "((r: Ref Int) => !r := 10)(ref 42)";
    let graph = parse_source(source, None)?;
    
    // The root should be an application
    let root_id = graph.root_id().unwrap();
    let root_node = graph.get_node(root_id).unwrap();
    assert_eq!(root_node.r#type, NodeType::TERM_APPLICATION as i32);
    
    // Get the application
    let app = graph.get_application(root_id)?;
    
    // The argument should be a reference creation
    let arg_id = app.argument_node_id;
    let arg_node = graph.get_node(arg_id).unwrap();
    assert_eq!(arg_node.r#type, NodeType::TERM_REF as i32);
    
    // The function body should be an assignment
    let func_id = app.function_node_id;
    let lambda = graph.get_lambda(func_id)?;
    let body_id = lambda.body_node_id;
    let body_node = graph.get_node(body_id).unwrap();
    assert_eq!(body_node.r#type, NodeType::TERM_ASSIGN as i32);
    
    Ok(())
}

#[test]
fn test_parse_effect() -> Result<()> {
    // perform('IO', 42)
    let source = "perform('IO', 42)";
    let graph = parse_source(source, None)?;
    
    // The root should be an effect performance
    let root_id = graph.root_id().unwrap();
    let root_node = graph.get_node(root_id).unwrap();
    assert_eq!(root_node.r#type, NodeType::EFFECT_PERFORM as i32);
    
    Ok(())
}