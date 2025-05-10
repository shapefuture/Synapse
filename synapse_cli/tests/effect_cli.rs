use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::path::Path;

#[test]
fn cli_effects_success_and_failure() {
    let src = r#"
        // Dummy code representing a node with IO effect
    "#;
    // Write temp ASG file with effect tag (simulate post-parsing)
    let effect_path = Path::new("effect.syn");
    let mut f = File::create(effect_path).unwrap();
    f.write_all(src.as_bytes()).unwrap();

    // --success (allow IO)--
    let ok = Command::new("cargo")
        .args(&["run", "--bin", "synapse_cli", "type-check-effects", effect_path.to_str().unwrap(), "--allow-effect", "IO"])
        .status()
        .unwrap();
    assert!(ok.success());

    // --failure (disallow IO)--
    let fail = Command::new("cargo")
        .args(&["run", "--bin", "synapse_cli", "type-check-effects", effect_path.to_str().unwrap(), "--allow-effect", "Pure"])
        .status()
        .unwrap();
    assert!(!fail.success());
}