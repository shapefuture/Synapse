//! Build script for synapse_ai_api: compiles protobuf definitions

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/synapseai.proto")?;
    Ok(())
}