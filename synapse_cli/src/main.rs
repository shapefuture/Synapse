mod compile;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(name = "synapse_cli", version)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile source file to executable
    Compile {
        input_file: PathBuf,
        #[clap(short, long)]
        output: PathBuf,
        #[clap(long, default_value = "target/debug/libsynapse_runtime.a")]
        runtime: PathBuf,
    }
    // ...other commands omitted for brevity...
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Compile { input_file, output, runtime } => {
            compile::compile_to_executable(&input_file, &output, &runtime)?;
        }
    }
    Ok(())
}