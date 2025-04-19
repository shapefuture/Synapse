use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {
    let proto_file = "../schemas/asg_schema_v1.proto";
    
    // Check if the proto file exists
    if !Path::new(proto_file).exists() {
        panic!("Proto file not found: {}", proto_file);
    }
    
    // Configure the output directory
    prost_build::Config::new()
        .out_dir("src/generated")
        .compile_protos(&[proto_file], &["../schemas"])?;
    
    // Inform Cargo to rerun this build script if the proto file changes
    println!("cargo:rerun-if-changed={}", proto_file);
    println!("cargo:rerun-if-changed=../schemas");
    
    Ok(())
}