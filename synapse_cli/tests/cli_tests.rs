use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Creates a temporary directory and returns its path.
fn setup_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}

/// Creates a test file with the given content.
fn create_test_file(dir: &TempDir, filename: &str, content: &str) -> PathBuf {
    let path = dir.path().join(filename);
    fs::write(&path, content).expect("Failed to write test file");
    path
}

/// Runs the synapse_cli command with the given arguments.
fn run_cli(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_synapse_cli"))
        .args(args)
        .output()
        .expect("Failed to execute synapse_cli");
    
    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    (success, stdout, stderr)
}

#[test]
fn test_parse_valid_file() {
    let dir = setup_temp_dir();
    let file_path = create_test_file(&dir, "valid.syn", "(x: Int) => x + 1");
    
    let (success, stdout, stderr) = run_cli(&["parse", file_path.to_str().unwrap()]);
    
    assert!(success);
    assert!(stdout.contains("Successfully parsed"));
    assert!(stderr.is_empty());
}

#[test]
fn test_parse_invalid_file() {
    let dir = setup_temp_dir();
    let file_path = create_test_file(&dir, "invalid.syn", "(x: Int => x + 1"); // Missing closing paren
    
    let (success, stdout, stderr) = run_cli(&["parse", file_path.to_str().unwrap()]);
    
    assert!(!success);
    assert!(stdout.is_empty());
    assert!(stderr.contains("ERROR"));
}

#[test]
fn test_format_valid_file() {
    let dir = setup_temp_dir();
    let file_path = create_test_file(&dir, "format.syn", "(x:Int)=>x+1"); // Unformatted
    let output_path = dir.path().join("formatted.syn");
    
    let (success, _, _) = run_cli(&[
        "format",
        file_path.to_str().unwrap(),
        "-o",
        output_path.to_str().unwrap(),
    ]);
    
    assert!(success);
    
    // Check the formatted output
    let formatted = fs::read_to_string(output_path).expect("Failed to read formatted file");
    assert_eq!(formatted, "(x: Int) => x + 1");
}

#[test]
fn test_lint_valid_file() {
    let dir = setup_temp_dir();
    let file_path = create_test_file(&dir, "lint_valid.syn", "(x: Int) => x + 1");
    
    let (success, stdout, _) = run_cli(&["lint", file_path.to_str().unwrap()]);
    
    assert!(success);
    assert!(stdout.contains("No lint errors found"));
}

#[test]
fn test_lint_invalid_file() {
    let dir = setup_temp_dir();
    let file_path = create_test_file(&dir, "lint_invalid.syn", "42(10)"); // Applying arguments to a non-function
    
    let (success, _, stderr) = run_cli(&["lint", file_path.to_str().unwrap()]);
    
    assert!(!success);
    assert!(stderr.contains("Cannot apply arguments to a non-function value"));
}

#[test]
fn test_dump_asg_json() {
    let dir = setup_temp_dir();
    let file_path = create_test_file(&dir, "dump.syn", "(x: Int) => x");
    let output_path = dir.path().join("dump.json");
    
    let (success, _, _) = run_cli(&[
        "dump-asg",
        file_path.to_str().unwrap(),
        "--format", "json",
        "-o",
        output_path.to_str().unwrap(),
    ]);
    
    assert!(success);
    
    // Check that the JSON file exists
    assert!(output_path.exists());
    
    // Basic check of JSON content
    let json = fs::read_to_string(output_path).expect("Failed to read JSON dump");
    assert!(json.contains("\"nodes\":"));
}

#[test]
fn test_dump_asg_binary() {
    let dir = setup_temp_dir();
    let file_path = create_test_file(&dir, "dump_bin.syn", "(x: Int) => x");
    let output_path = dir.path().join("dump.asg");
    
    let (success, _, _) = run_cli(&[
        "dump-asg",
        file_path.to_str().unwrap(),
        "--format", "binary",
        "-o",
        output_path.to_str().unwrap(),
    ]);
    
    assert!(success);
    
    // Check that the binary file exists
    assert!(output_path.exists());
}