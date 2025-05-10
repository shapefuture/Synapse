use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::path::Path;

#[test]
fn adt_poly_effect_cli_pipeline() {
    // Writes a stub (see plan: real parsing in future release)
    let src = r#"
        // demo: polymorphic Option<T> with IO effect
    "#;
    let path = Path::new("test_polyadt.syn");
    let mut f = File::create(path).unwrap();
    f.write_all(src.as_bytes()).unwrap();

    // Lower to UPIR and check output
    let upir = Command::new("cargo")
        .args(&["run", "--bin", "synapse_cli", "lower-upir", path.to_str().unwrap()])
        .output()
        .unwrap();
    let upir_out = String::from_utf8_lossy(&upir.stdout);
    assert!(upir.status.success());
    assert!(upir_out.contains("Option") || upir_out.contains("core.match"));

    // Type/effect check: should work if IO allowed
    let tc = Command::new("cargo")
        .args(&["run", "--bin", "synapse_cli", "type-check-effects", path.to_str().unwrap(), "--allow-effect", "IO"])
        .status()
        .unwrap();
    assert!(tc.success());

    // Should fail if IO effect not allowed
    let tc_fail = Command::new("cargo")
        .args(&["run", "--bin", "synapse_cli", "type-check-effects", path.to_str().unwrap(), "--allow-effect", "Pure"])
        .status()
        .unwrap();
    assert!(!tc_fail.success());
}