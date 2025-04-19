//! Synapse CLI is the main command-line interface for the Synapse language.

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use asg_core::{AsgGraph, save_asg_binary, save_asg_json};
use clap::{Parser, Subcommand};
use colored::*;
use formatter_core::format_asg;
use parser_core::parse_file;

mod linter;
use linter::{lint_graph, LintError};

/// CLI argument parser using clap.
#[derive(Parser)]
#[command(
    name = "synapse_cli",
    version = "0.1.0",
    about = "Command-line interface for the Synapse language",
    long_about = "Synapse CLI provides tools for working with the Synapse programming language."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Supported CLI subcommands.
#[derive(Subcommand)]
enum Commands {
    /// Parse a Synapse source file
    Parse {
        /// Input file to parse
        input_file: PathBuf,
    },
    
    /// Format a Synapse source file
    Format {
        /// Input file to format
        input_file: PathBuf,
        
        /// Output file (stdout if not specified)
        #[clap(short, long)]
        output_file: Option<PathBuf>,
    },
    
    /// Lint a Synapse source file
    Lint {
        /// Input file to lint
        input_file: PathBuf,
    },
    
    /// Dump the ASG of a Synapse source file
    DumpAsg {
        /// Input file to dump
        input_file: PathBuf,
        
        /// Output format
        #[clap(long, default_value = "binary")]
        format: DumpFormat,
        
        /// Output file (stdout if not specified, except for binary format)
        #[clap(short, long)]
        output_file: Option<PathBuf>,
    },
}

/// Supported output formats for ASG dumping.
#[derive(clap::ValueEnum, Clone)]
enum DumpFormat {
    Binary,
    Json,
}

/// Error type for CLI operations.
#[derive(thiserror::Error, Debug)]
enum CliError {
    #[error("Parser error: {0}")]
    Parser(#[from] parser_core::error::ParseError),
    
    #[error("Formatter error: {0}")]
    Formatter(#[from] formatter_core::FormatError),
    
    #[error("ASG error: {0}")]
    Asg(#[from] asg_core::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Lint errors: {0} issues found")]
    Lint(usize),
    
    #[error("General error: {0}")]
    General(String),
}

type Result<T> = std::result::Result<T, CliError>;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Parse { input_file } => {
            match parse_file(input_file) {
                Ok(graph) => {
                    println!("{} Successfully parsed {}", "SUCCESS:".green(), input_file.display());
                    println!("Graph contains {} nodes", graph.nodes().len());
                    if let Some(root_id) = graph.root_id() {
                        println!("Root node ID: {}", root_id);
                    } else {
                        println!("No root node set");
                    }
                }
                Err(err) => {
                    eprintln!("{} {}", "ERROR:".red(), err);
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Format { input_file, output_file } => {
            format_command(input_file, output_file)?;
        }
        
        Commands::Lint { input_file } => {
            lint_command(input_file)?;
        }
        
        Commands::DumpAsg { input_file, format, output_file } => {
            dump_asg_command(input_file, format, output_file)?;
        }
    }
    
    Ok(())
}

/// Handles the Format command.
fn format_command(input_file: &PathBuf, output_file: &Option<PathBuf>) -> Result<()> {
    // Parse the input file
    let graph = parse_file(input_file)?;
    
    // Get the root node
    let root_id = graph.root_id().ok_or_else(|| {
        CliError::General("No root node found in the ASG".to_string())
    })?;
    
    // Format the ASG
    let formatted = format_asg(&graph, root_id)?;
    
    // Output the formatted code
    match output_file {
        Some(path) => {
            fs::write(path, formatted)?;
            println!("{} Formatted output written to {}", "SUCCESS:".green(), path.display());
        }
        None => {
            // Print to stdout
            println!("{}", formatted);
        }
    }
    
    Ok(())
}

/// Handles the Lint command.
fn lint_command(input_file: &PathBuf) -> Result<()> {
    // Parse the input file
    let graph = parse_file(input_file)?;
    
    // Run the linter
    let errors = lint_graph(&graph);
    
    // Report errors
    if errors.is_empty() {
        println!("{} No lint errors found", "SUCCESS:".green());
        Ok(())
    } else {
        print_lint_errors(&errors);
        Err(CliError::Lint(errors.len()))
    }
}

/// Handles the DumpAsg command.
fn dump_asg_command(input_file: &PathBuf, format: &DumpFormat, output_file: &Option<PathBuf>) -> Result<()> {
    // Parse the input file
    let graph = parse_file(input_file)?;
    
    match format {
        DumpFormat::Binary => {
            // Binary format always requires an output file
            let output_path = output_file.clone().ok_or_else(|| {
                CliError::General("Binary format requires an output file".to_string())
            })?;
            
            save_asg_binary(&graph, output_path)?;
            println!("{} Binary ASG written to {}", "SUCCESS:".green(), output_path.display());
        }
        DumpFormat::Json => {
            match output_file {
                Some(path) => {
                    save_asg_json(&graph, path)?;
                    println!("{} JSON ASG written to {}", "SUCCESS:".green(), path.display());
                }
                None => {
                    // Create JSON string and print to stdout
                    let proto_graph = graph.clone().into_proto();
                    let json = serde_json::to_string_pretty(&proto_graph)
                        .map_err(|e| CliError::General(format!("JSON serialization error: {}", e)))?;
                    println!("{}", json);
                }
            }
        }
    }
    
    Ok(())
}

/// Prints lint errors to stderr with colored output.
fn print_lint_errors(errors: &[LintError]) {
    eprintln!("{} {} lint errors found:", "ERROR:".red(), errors.len());
    
    for error in errors {
        let code = error.code.yellow();
        let message = error.message.red();
        
        if let Some(loc) = &error.source_location {
            eprintln!(
                "  [{}] {} at {}:{}:{}",
                code, message, loc.filename, loc.start_line, loc.start_col
            );
        } else {
            eprintln!("  [{}] {} at node {}", code, message, error.node_id);
        }
    }
}