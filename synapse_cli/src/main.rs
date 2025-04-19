/// Synapse CLI: User interface for the Synapse compiler & tools.
/// 
/// Subcommands: parse, format, lint, dump-asg.
/// Structure and logic aligned with the Synapse implementation plan.

mod linter;

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueEnum};

use asg_core::AsgGraph;
use parser_core;
use formatter_core;
use linter::{lint_graph, LintError};

#[derive(Parser)]
#[command(
    name = "synapse_cli",
    about = "Command line interface for the Synapse language.",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a Synapse source file and check for syntax errors
    Parse {
        input_file: PathBuf,
    },
    /// Pretty-print Synapse source file with canonical formatting
    Format {
        input_file: PathBuf,
        /// Write output to file instead of stdout
        #[arg(short, long)]
        output_file: Option<PathBuf>,
    },
    /// Lint a source file for structural and semantic issues
    Lint {
        input_file: PathBuf,
    },
    /// Dump the ASG (Abstract Semantic Graph) in binary or JSON
    DumpAsg {
        input_file: PathBuf,
        #[arg(long, value_enum, default_value_t = DumpFormat::Binary)]
        format: DumpFormat,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum DumpFormat {
    Binary,
    Json,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { input_file } => {
            let src = std::fs::read_to_string(&input_file)?;
            match parser_core::parse_str(&src) {
                Ok(_graph) => {
                    println!("Parse successful: {input_file:?}");
                    Ok(())
                }
                Err(err) => {
                    eprintln!("Parse failed:\n{err}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Format { input_file, output_file } => {
            let src = std::fs::read_to_string(&input_file)?;
            let graph = parser_core::parse_str(&src)
                .map_err(|e| anyhow::anyhow!("Parse failed: {e}"))?;
            let formatted = formatter_core::format_asg(&graph, graph.root_id())
                .map_err(|e| anyhow::anyhow!("Formatting failed: {e}"))?;
            match output_file {
                Some(outfile) => {
                    std::fs::write(&outfile, formatted)?;
                }
                None => {
                    print!("{formatted}");
                }
            }
            Ok(())
        }
        Commands::Lint { input_file } => {
            let src = std::fs::read_to_string(&input_file)?;
            let graph = parser_core::parse_str(&src)
                .map_err(|e| anyhow::anyhow!("Parse failed: {e}"))?;
            let warnings = lint_graph(&graph);
            if warnings.is_empty() {
                println!("No lint errors found.");
                Ok(())
            } else {
                for e in &warnings {
                    eprintln!("[{}] {} (node_id={})", e.code, e.message, e.node_id);
                    if let Some(loc) = &e.source_location {
                        eprintln!("  at {}:{}-{}:{}", loc.filename, loc.start_line, loc.end_line, loc.end_col);
                    }
                }
                std::process::exit(2);
            }
        }
        Commands::DumpAsg { input_file, format } => {
            let src = std::fs::read_to_string(&input_file)?;
            let graph = parser_core::parse_str(&src)
                .map_err(|e| anyhow::anyhow!("Parse failed: {e}"))?;
            match format {
                DumpFormat::Binary => {
                    let out: Vec<u8> = asg_core::serialize_to_binary(&graph)
                        .map_err(|e| anyhow::anyhow!("ASG binary serialization error: {e}"))?;
                    // Write to stdout as binary
                    let mut stdout = io::stdout();
                    stdout.write_all(&out)?;
                }
                DumpFormat::Json => {
                    // Optionally, implement JSON serialization if available in asg_core.
                    let json = serde_json::to_string_pretty(&graph)
                        .map_err(|e| anyhow::anyhow!("ASG JSON serialization error: {e}"))?;
                    println!("{json}");
                }
            }
            Ok(())
        }
    }
}