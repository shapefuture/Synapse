//! Synapse AI Tutor: Interactive learning assistant for Synapse language
//! 
//! Features:
//! - Explain Synapse concepts
//! - Analyze and explain code errors
//! - Suggest fixes for common errors
//! - Answer questions about language features

use clap::{Parser, Subcommand};
use rustyline::{Editor, error::ReadlineError};
use colored::*;
use parser_core::parse_str;
use type_checker_l2::check_and_annotate_graph_v2_with_effects_check;
use proof_synthesis_assist::{explain_type_error, explain_effect_error};

/// Command-line arguments
#[derive(Parser)]
#[clap(name = "synapse_tutor")]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start interactive REPL session
    Repl,
    
    /// Explain a specific error message
    Explain {
        /// Error message to explain
        #[clap(required = true)]
        error: String,
    },
    
    /// Explain a language concept
    Concept {
        /// Concept to explain (e.g., "effects", "linearity", "dependent-types")
        #[clap(required = true)]
        concept: String,
    },
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Repl) => run_repl(),
        Some(Commands::Explain { error }) => explain_error(&error),
        Some(Commands::Concept { concept }) => explain_concept(&concept),
        None => run_repl(),
    }
}

/// Run the interactive REPL
fn run_repl() {
    println!("{}", "Welcome to Synapse AI Tutor!".green().bold());
    println!("Type {} to exit. Enter Synapse code, errors, or ask questions.", "exit".yellow());
    println!("{}", "Examples:".cyan());
    println!("  - Analyze: lambda (x: Int) -> x + 1");
    println!("  - Explain: UnificationFail(Int, Bool)");
    println!("  - Ask: What are effect capabilities?");
    println!("  - Command: /help");
    
    let mut rl = Editor::<()>::new();
    
    loop {
        let readline = rl.readline("synapse> ");
        match readline {
            Ok(line) => {
                if line.trim() == "exit" || line.trim() == "quit" {
                    break;
                }
                
                rl.add_history_entry(line.as_str());
                process_input(&line);
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}

/// Process user input (either code, error, question, or command)
fn process_input(input: &str) {
    let trimmed = input.trim();
    
    // Handle commands (start with /)
    if trimmed.starts_with('/') {
        handle_command(&trimmed[1..]);
        return;
    }
    
    // Check if it looks like a question
    if trimmed.ends_with('?') || 
       trimmed.to_lowercase().starts_with("what") ||
       trimmed.to_lowercase().starts_with("how") ||
       trimmed.to_lowercase().starts_with("why") ||
       trimmed.to_lowercase().starts_with("explain") {
        answer_question(trimmed);
        return;
    }
    
    // Check if it looks like an error message
    if trimmed.contains("Error") || 
       trimmed.contains("UnificationFail") || 
       trimmed.contains("TypeError") {
        explain_error(trimmed);
        return;
    }
    
    // Otherwise, treat it as Synapse code and analyze it
    analyze_code(trimmed);
}

/// Handle a command (starting with /)
fn handle_command(cmd: &str) {
    match cmd.trim() {
        "help" => {
            println!("{}", "Commands:".green().bold());
            println!("  /help           - Show this help");
            println!("  /concepts       - List available concepts");
            println!("  /examples       - Show example Synapse code");
            println!("  /errors         - List common error types");
        },
        "concepts" => {
            println!("{}", "Available concepts:".green().bold());
            println!("  - effects        - Effect system and capabilities");
            println!("  - linearity      - Linear/affine types for resource management");
            println!("  - dependent      - Dependent types for verification");
            println!("  - polymorphism   - Parametric polymorphism (System F)");
            println!("  - pattern-match  - Pattern matching and algebraic data types");
        },
        "examples" => {
            println!("{}", "Example Synapse code:".green().bold());
            println!("  lambda (x: Int) -> x + 1");
            println!("  data Option = Some(Int) | None");
            println!("  let add = lambda (x: Int) (y: Int) -> x + y");
            println!("  perform IO \"Hello, world!\"");
        },
        "errors" => {
            println!("{}", "Common error types:".green().bold());
            println!("  UnificationFail(Type1, Type2) - Type mismatch");
            println!("  OccursCheck(Var, Type)        - Recursive type");
            println!("  UndefinedVariable(NodeId)     - Variable not in scope");
            println!("  Effect 'IO' not allowed       - Effect capability error");
        },
        _ => println!("Unknown command: /{}", cmd),
    }
}

/// Answer a user question
fn answer_question(question: &str) {
    let lower_q = question.to_lowercase();
    
    // Very basic pattern matching
    if lower_q.contains("effect") {
        explain_concept("effects");
    } else if lower_q.contains("linear") || lower_q.contains("affine") {
        explain_concept("linearity");
    } else if lower_q.contains("dependent") || lower_q.contains("verification") {
        explain_concept("dependent");
    } else if lower_q.contains("polymorphism") || lower_q.contains("generics") {
        explain_concept("polymorphism");
    } else if lower_q.contains("pattern") || lower_q.contains("match") || lower_q.contains("adt") {
        explain_concept("pattern-match");
    } else {
        println!("{}", "I'm not sure how to answer that question.".yellow());
        println!("Try asking about specific language features like effects, linearity, or dependent types.");
        println!("Or use /concepts to see available topics.");
    }
}

/// Explain a specific error message
fn explain_error(error: &str) {
    let lower_error = error.to_lowercase();
    
    if lower_error.contains("unification") || lower_error.contains("type mismatch") {
        // Simulate a unification error
        println!("{}", "Type Mismatch Error Explanation:".green().bold());
        println!("This error occurs when the type checker can't reconcile two different types that need to be the same.");
        println!("For example, if you have an expression like:");
        println!("  let x: Bool = 42");
        println!("The type checker expects a Bool but finds an Int (42).");
        println!();
        println!("{}", "Suggested fix:".cyan().bold());
        println!("Make sure the types match on both sides of the expression. Either:");
        println!("1. Change the type annotation: let x: Int = 42");
        println!("2. Or provide a Bool value: let x: Bool = true");
    }
    else if lower_error.contains("effect") {
        // Simulate an effect capability error
        let result = explain_effect_error(&["Pure".to_string()], "IO");
        if let Ok(explanation) = result {
            println!("{}", "Effect Error Explanation:".green().bold());
            println!("{}", explanation.explanation);
            println!();
            if let Some(fix) = explanation.suggested_fix {
                println!("{}", "Suggested fix:".cyan().bold());
                println!("{}", fix);
            }
            if let Some(code) = explanation.code_fix {
                println!();
                println!("{}", code);
            }
        }
    }
    else {
        println!("{}", "I'm not sure how to explain this error.".yellow());
        println!("Try using a more specific error message or looking at common errors with /errors.");
    }
}

/// Explain a language concept
fn explain_concept(concept: &str) {
    let lower_concept = concept.to_lowercase();
    
    match lower_concept.as_str() {
        "effects" | "effect" => {
            println!("{}", "Effect System in Synapse".green().bold());
            println!("The effect system tracks and controls side effects in your code.");
            println!();
            println!("Key points:");
            println!("- Effects are capabilities a function needs (IO, State, Exception)");
            println!("- Pure functions have no effects");
            println!("- Effect tags on functions declare what effects they can perform");
            println!("- The type checker ensures you don't use effects you don't have permission for");
            println!();
            println!("Example:");
            println!("  # This requires the IO effect capability");
            println!("  perform IO \"Hello, world!\"");
            println!();
            println!("  # This declares that the function needs IO permission");
            println!("  fn greet() with [IO] {");
            println!("    perform IO \"Hello!\"");
            println!("  }");
        },
        "linearity" | "linear" | "affine" => {
            println!("{}", "Linear and Affine Types in Synapse".green().bold());
            println!("These are quantitative types that control how values can be used:");
            println!();
            println!("Linear types: Must be used exactly once (no duplication, no dropping)");
            println!("Affine types: Can be used at most once (can be dropped, not duplicated)");
            println!();
            println!("They provide memory and resource safety without garbage collection.");
            println!();
            println!("Example:");
            println!("  # FileHandle is a linear type (must be closed exactly once)");
            println!("  let f = open_file(\"data.txt\")  # Linear");
            println!("  let contents = read(f)          # Consumes f");
            println!("  # f is no longer available here");
        },
        "dependent" | "dependent-types" => {
            println!("{}", "Dependent Types in Synapse".green().bold());
            println!("Dependent types allow types to depend on values, enabling precise specifications.");
            println!();
            println!("They can express properties like:");
            println!("- Arrays with their length in the type (Vector<T, n>)");
            println!("- Functions that only accept positive numbers");
            println!("- Types that ensure sorting, balance, or other invariants");
            println!();
            println!("Example:");
            println!("  # Type Vector depends on value n");
            println!("  fn append<T, n: Nat, m: Nat>(v1: Vector<T, n>, v2: Vector<T, m>) -> Vector<T, n+m>");
            println!();
            println!("  # The type system verifies the property statically");
            println!("  let v: Vector<Int, 3> = [1, 2, 3]");
        },
        "polymorphism" | "generics" => {
            println!("{}", "Parametric Polymorphism in Synapse".green().bold());
            println!("Synapse uses System F-style parametric polymorphism, allowing code that works with any type.");
            println!();
            println!("Key aspects:");
            println!("- Type variables (forall X. ...)");
            println!("- Generic functions and data structures");
            println!("- Type inference with type variable instantiation");
            println!();
            println!("Example:");
            println!("  # Generic identity function (works for any type X)");
            println!("  let id = lambda <X> (x: X) -> x");
            println!();
            println!("  # Usage with different types");
            println!("  id<Int>(42)");
            println!("  id<Bool>(true)");
        },
        "pattern-match" | "pattern" | "adt" => {
            println!("{}", "Pattern Matching and ADTs in Synapse".green().bold());
            println!("Algebraic Data Types (ADTs) and pattern matching provide powerful data modeling.");
            println!();
            println!("ADTs can be:");
            println!("- Sum types (variants/tagged unions)");
            println!("- Product types (records/structs)");
            println!();
            println!("Pattern matching allows:");
            println!("- Destructuring data");
            println!("- Exhaustiveness checking (compiler ensures all cases handled)");
            println!("- Guard conditions");
            println!();
            println!("Example:");
            println!("  # Define an ADT");
            println!("  data Result<T, E> = Ok(T) | Error(E)");
            println!();
            println!("  # Pattern matching");
            println!("  match result {");
            println!("    Ok(value) => /* use value */,");
            println!("    Error(err) => /* handle error */");
            println!("  }");
        },
        _ => {
            println!("{}", "I don't have information about that concept yet.".yellow());
            println!("Try /concepts to see the available topics.");
        }
    }
}

/// Analyze Synapse code and provide feedback
fn analyze_code(code: &str) {
    println!("{}", "Analyzing code:".green().bold());
    println!("{}", code);
    println!();
    
    // Try to parse the code
    match parse_str(code) {
        Ok(mut asg) => {
            println!("{}", "✓ Parsing successful".green());
            
            // Try to type check
            match check_and_annotate_graph_v2_with_effects_check(&mut asg, &[] as &[&str]) {
                Ok(_) => {
                    println!("{}", "✓ Type checking successful".green());
                    println!();
                    println!("Code is valid Synapse! Here's an explanation:");
                    
                    // Basic code explanation based on structure
                    if code.contains("lambda") {
                        println!("This code defines a function using lambda expression.");
                        if code.contains("->") {
                            let parts: Vec<&str> = code.split("->").collect();
                            if parts.len() >= 2 {
                                println!("It takes parameters from the left side of -> and returns the expression on the right.");
                            }
                        }
                    }
                    else if code.contains("let") {
                        println!("This code defines a value binding using let.");
                    }
                    else if code.contains("data") {
                        println!("This code defines an algebraic data type with constructors.");
                    }
                    
                    println!();
                    println!("{}", "Note: This is a simplified analysis. For detailed type and effect information, use the LSP in your editor.".yellow());
                },
                Err(err) => {
                    println!("{}", "✗ Type checking failed".red());
                    println!("{}", format!("Error: {}", err).red());
                    println!();
                    
                    // Try to explain the error
                    match explain_type_error(&err, Some(code)) {
                        Ok(explanation) => {
                            println!("{}", "Error explanation:".yellow().bold());
                            println!("{}", explanation.explanation);
                            
                            if let Some(fix) = explanation.suggested_fix {
                                println!();
                                println!("{}", "Suggested fix:".cyan().bold());
                                println!("{}", fix);
                            }
                            
                            if let Some(code_fix) = explanation.code_fix {
                                println!();
                                println!("{}", code_fix);
                            }
                        },
                        Err(_) => {
                            println!("I'm not sure how to explain this error in detail.");
                        }
                    }
                }
            }
        },
        Err(err) => {
            println!("{}", "✗ Parsing failed".red());
            println!("{}", format!("Error: {}", err).red());
            println!();
            println!("{}", "Make sure your syntax is correct. Here are some examples:".yellow());
            println!("  lambda (x: Int) -> x + 1");
            println!("  let x = 42");
            println!("  data Option = Some(Int) | None");
        }
    }
}