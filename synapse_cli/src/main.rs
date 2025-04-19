//! Synapse CLI: Command-line interface for the Synapse language.
//! 
//! Provides subcommands for parsing, formatting, linting, and inspecting
//! Synapse source code using the Abstract Semantic Graph (ASG).

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use fs_err as fs;

use asg_core::AsgGraph;
use parser_core::parse_text;
use formatter_core::format_asg;

mod linter;
use linter::{lint_graph, LintError};

/// Synapse language compiler and toolchain
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse Synapse source and check for syntax errors
    Parse {
        /// Input file to parse
        input_file: PathBuf,
    },
    
    /// Format Synapse code in canonical form
    Format {
        /// Input file to format
        input_file: PathBuf,
        
        /// Output file (defaults to stdout if not provided)
        #[clap(short, long)]
        output_file: Option<PathBuf>,
    },
    
    /// Lint Synapse code for structural errors
    Lint {
        /// Input file to lint
        input_file: PathBuf,
    },
    
    /// Dump the ASG in binary or JSON format
    DumpAsg {
        /// Input file to dump ASG for
        input_file: PathBuf,
        
        /// Output format
        #[clap(long, arg_enum, default_value = "json")]
        format: DumpFormat,
    },
}

#[derive(clap::ArgEnum, Clone)]
enum DumpFormat {
    Binary,
    Json,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Parse { input_file } => {
            handle_parse(input_file)
        },
        Commands::Format { input_file, output_file } => {
            handle_format(input_file, output_file)
        },
        Commands::Lint { input_file } => {
            handle_lint(input_file)
        },
        Commands::DumpAsg { input_file, format } => {
            handle_dump_asg(input_file, format)
        },
    }
}

fn handle_parse(input_file: &PathBuf) -> Result<()> {
    let source = fs::read_to_string(input_file)
        .with_context(|| format!("Failed to read input file: {}", input_file.display()))?;
    
    match parse_text(&source) {
        Ok(_) => {
            println!("{}", "Parse successful.".green());
            Ok(())
        },
        Err(err) => {
            eprintln!("{}: {}", "Parse error".bright_red(), err);
            process::exit(1);
        }
    }
}

fn handle_format(input_file: &PathBuf, output_file: &Option<PathBuf>) -> Result<()> {
    // Read the input file
    let source = fs::read_to_string(input_file)
        .with_context(|| format!("Failed to read input file: {}", input_file.display()))?;
    
    // Parse the source into an ASG
    let asg = parse_text(&source)
        .with_context(|| format!("Failed to parse input file: {}", input_file.display()))?;
    
    // Format the ASG back to text
    let formatted = format_asg(&asg, asg.root_node_id())
        .with_context(|| "Failed to format ASG")?;
    
    // Output the formatted text
    match output_file {
        Some(path) => {
            fs::write(path, formatted)
                .with_context(|| format!("Failed to write to output file: {}", path.display()))?;
            println!("Formatted output written to: {}", path.display());
        },
        None => {
            println!("{}", formatted);
        }
    }
    
    Ok(())
}

fn handle_lint(input_file: &PathBuf) -> Result<()> {
    // Read the input file
    let source = fs::read_to_string(input_file)
        .with_context(|| format!("Failed to read input file: {}", input_file.display()))?;
    
    // Parse the source into an ASG
    let asg = parse_text(&source)
        .with_context(|| format!("Failed to parse input file: {}", input_file.display()))?;
    
    // Run the linter
    let lint_errors = lint_graph(&asg);
    
    // Print results
    if lint_errors.is_empty() {
        println!("{}", "No lint errors found.".green());
        Ok(())
    } else {
        eprintln!("{} {} found:", lint_errors.len(), 
                 if lint_errors.len() == 1 {"lint error"} else {"lint errors"});
        
        for error in lint_errors {
            print_lint_error(&error, input_file);
        }
        
        // Exit with error code for CI/pipelines
        process::exit(2);
    }
}

fn handle_dump_asg(input_file: &PathBuf, format: &DumpFormat) -> Result<()> {
    // Read the input file
    let source = fs::read_to_string(input_file)
        .with_context(|| format!("Failed to read input file: {}", input_file.display()))?;
    
    // Parse the source into an ASG
    let asg = parse_text(&source)
        .with_context(|| format!("Failed to parse input file: {}", input_file.display()))?;
    
    // Output the ASG in the requested format
    match format {
        DumpFormat::Binary => {
            // For a simple first version, we'll serialize to a binary format via JSON
            // In a real implementation, you would use asg_core's binary serialization
            let serialized = serde_json::to_vec(&asg)
                .with_context(|| "Failed to serialize ASG to binary")?;
            io::stdout().write_all(&serialized)?;
        },
        DumpFormat::Json => {
            // Pretty-print the ASG as JSON
            let json = serde_json::to_string_pretty(&asg)
                .with_context(|| "Failed to serialize ASG to JSON")?;
            println!("{}", json);
        }
    }
    
    Ok(())
}

fn print_lint_error(error: &LintError, file_path: &PathBuf) {
    let error_code = format!("[{}]", error.code).bright_red();
    
    if let Some(location) = &error.location {
        eprintln!("{}:{}: {} {}: {}",
                 location.filename.as_deref().unwrap_or(file_path.to_str().unwrap_or("unknown")),
                 location.start_line,
                 error_code,
                 "error".bright_red(),
                 error.message);
    } else {
        eprintln!("{} {}: {} (node: {})",
                 error_code,
                 "error".bright_red(),
                 error.message,
                 error.node_id);
    }
}