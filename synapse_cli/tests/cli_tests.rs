//! Integration tests for the Synapse CLI.
//!
//! Tests for basic CLI commands: parse, format, lint, dump-asg.

use std::fs;
use std::path::PathBuf;
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

const VALID_EXAMPLE: &str = r#"
lambda (x: Int) -> x + 1
"#;

const INVALID_EXAMPLE: &str = r#"
lambda (x: Int) -> x +
"#;

const LINT_ERROR_EXAMPLE: &str = r#"
lambda (x: Int) -> 
    let y = 5 in
    let z = ref y in
    z := 3 + true  // Type error: adding boolean to integer
"#;

// Helper to set up a temp file with content
fn setup_temp_file(content: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.syn");
    fs::write(&file_path, content).unwrap();
    (temp_dir, file_path)
}

#[test]
fn test_parse_valid() {
    let (_temp_dir, file_path) = setup_temp_file(VALID_EXAMPLE);
    
    Command::cargo_bin("synapse_cli")
        .unwrap()
        .arg("parse")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Parse successful"));
}

#[test]
fn test_parse_invalid() {
    let (_temp_dir, file_path) = setup_temp_file(INVALID_EXAMPLE);
    
    Command::cargo_bin("synapse_cli")
        .unwrap()
        .arg("parse")
        .arg(file_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Parse error"));
}

#[test]
fn test_format() {
    let (temp_dir, file_path) = setup_temp_file(VALID_EXAMPLE);
    let output_path = temp_dir.path().join("output.syn");
    
    Command::cargo_bin("synapse_cli")
        .unwrap()
        .arg("format")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();
    
    // Check output file exists
    assert!(output_path.exists());
    
    // The content should be formatted but semantically equivalent
    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("lambda"));
    assert!(content.contains("Int"));
    assert!(content.contains("+"));
}

#[test]
fn test_lint_valid() {
    let (_temp_dir, file_path) = setup_temp_file(VALID_EXAMPLE);
    
    Command::cargo_bin("synapse_cli")
        .unwrap()
        .arg("lint")
        .arg(file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No lint errors found"));
}

#[test]
fn test_lint_errors() {
    let (_temp_dir, file_path) = setup_temp_file(LINT_ERROR_EXAMPLE);
    
    // Note: Our linter implementation is fairly basic and may not catch all the errors
    // in this example at Level 0, but the test should run without crashing.
    Command::cargo_bin("synapse_cli")
        .unwrap()
        .arg("lint")
        .arg(file_path)
        .assert()
        .failure();
}

#[test]
fn test_dump_asg_json() {
    let (_temp_dir, file_path) = setup_temp_file(VALID_EXAMPLE);
    
    Command::cargo_bin("synapse_cli")
        .unwrap()
        .arg("dump-asg")
        .arg(file_path)
        .arg("--format=json")
        .assert()
        .success()
        .stdout(predicate::str::contains("nodes"));
}

#[test]
fn test_nonexistent_file() {
    Command::cargo_bin("synapse_cli")
        .unwrap()
        .arg("parse")
        .arg("nonexistent_file.syn")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to read input file"));
}