use asg_core::AsgGraph;
use formatter_core::{format_asg, Result};
use parser_core::parse_source;

/// Helper function to test round-trip parsing and formatting
fn test_round_trip(source: &str) -> Result<()> {
    // Parse the source into an ASG
    let graph = parse_source(source, None).unwrap();
    
    // Format the ASG back to source
    let root_id = graph.root_id().unwrap();
    let formatted = format_asg(&graph, root_id)?;
    
    // Parse the formatted source again
    let reparsed_graph = parse_source(&formatted, None).unwrap();
    
    // Format again for comparison
    let reparsed_root_id = reparsed_graph.root_id().unwrap();
    let reformatted = format_asg(&reparsed_graph, reparsed_root_id)?;
    
    // The two formatted strings should be identical
    assert_eq!(formatted, reformatted);
    
    Ok(())
}

#[test]
fn test_format_lambda() -> Result<()> {
    let source = "(x: Int) => x + 1";
    test_round_trip(source)
}

#[test]
fn test_format_application() -> Result<()> {
    let source = "((x: Int) => x + 1)(42)";
    test_round_trip(source)
}

#[test]
fn test_format_nested_lambdas() -> Result<()> {
    let source = "(x: Int) => (y: Int) => x + y";
    test_round_trip(source)
}

#[test]
fn test_format_arithmetic() -> Result<()> {
    let source = "1 + 2 * 3 - 4 / 5";
    test_round_trip(source)
}

#[test]
fn test_format_comparisons() -> Result<()> {
    let source = "1 < 2 && 3 > 4 || 5 == 6";
    test_round_trip(source)
}

#[test]
fn test_format_references() -> Result<()> {
    let source = "!ref 42 := 10";
    test_round_trip(source)
}

#[test]
fn test_format_effect() -> Result<()> {
    let source = "perform('IO', 42)";
    test_round_trip(source)
}

#[test]
fn test_format_complex_types() -> Result<()> {
    let source = "(f: Int -> Int) => (x: Int) => f(x) + f(x)";
    test_round_trip(source)
}

#[test]
fn test_format_ref_types() -> Result<()> {
    let source = "(r: Ref Int) => !r";
    test_round_trip(source)
}