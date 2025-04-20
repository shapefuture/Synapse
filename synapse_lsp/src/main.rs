//! Synapse LSP++ entrypoint (Phase 3.1): diagnostics, hover, minimal completion.

mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    server::run_lsp_server().await?;
    Ok(())
}