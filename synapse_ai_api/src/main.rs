//! Core AI API server for programmatic ASG/type/query access.
//! Provides proto/gRPC for parse/check/query functionality (P3T2).

mod proto;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server::run_grpc_server().await?;
    Ok(())
}