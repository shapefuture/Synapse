use std::process::Command;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[test]
fn end_to_end_compile_prints_42() {
    // Write a minimal Synapse source file "lambda (_: Int) -> 42"
    let src = "(lambda (x: Int) 42)";
    let src_path = Path::new("test_42.syn");
    let mut f = File::create(src_path).unwrap();
    f.write_all(src.as_bytes()).unwrap();

    // Compile to executable
    let exe_path = Path::new("test_42_exec");
    let runtime_path = Path::new("target/debug/libsynapse_runtime.a"); // Adjust for release if needed
    let status = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("synapse_cli")
        .arg("compile")
        .arg(src_path)
        .arg("-o")
        .arg(exe_path)
        .arg("--runtime")
        .arg(runtime_path)
        .status()
        .unwrap();
    assert!(status.success());

    // Run output and check printout
    let out = Command::new(exe_path).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("42"));
}