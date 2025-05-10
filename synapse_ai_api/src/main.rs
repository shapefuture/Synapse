//! Synapse AI API server entrypoint - provides gRPC interface for programmatic
//! ASG parsing, type checking, and query functionality for AI assistants.

mod server;

// Import proto-generated code
pub mod proto {
    tonic::include_proto!("synapseai");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Synapse AI API server...");
    server::run_grpc_server().await?;
    Ok(())
}