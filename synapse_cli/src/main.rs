mod compile;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use parser_core::parse_file;
use type_checker_l2::{check_and_annotate_graph_v2_with_effects_check, TypeError};
use asg_to_upir::lower_graph_to_upir;
use upir_core::ir::print_module;

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
    },
    /// Type check a source file for ADTs/polymorphism/effects
    TypeCheckEffects {
        input_file: PathBuf,
        #[clap(long, use_value_delimiter = true, value_delimiter = ',')]
        allow_effect: Vec<String>,
    },
    /// Lower polymorphic/ADT ASG to UPIR
    LowerUpir {
        input_file: PathBuf,
    }
    // ...other commands omitted for brevity...
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Compile { input_file, output, runtime } => {
            compile::compile_to_executable(&input_file, &output, &runtime)?;
        }
        Commands::TypeCheckEffects { input_file, allow_effect } => {
            let mut asg = parse_file(&input_file)?;
            match check_and_annotate_graph_v2_with_effects_check(&mut asg, &allow_effect.iter().map(|s| s.as_str()).collect::<Vec<_>>()) {
                Ok(()) => {
                    println!("Type/effect check passed.");
                }
                Err(e) => {
                    eprintln!("Type/effect check error: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::LowerUpir { input_file } => {
            let mut asg = parse_file(&input_file)?;
            let upir_mod = lower_graph_to_upir(&asg)?;
            println!("{}", print_module(&upir_mod));
        }
    }
    Ok(())
}