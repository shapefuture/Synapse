# Synapse Implementation Plan (v3) - Detailed Steps

Objective: To implement the Synapse language, compiler, runtime, and tooling according to the "Revised Ultimate Proposal: Synapse (v3)".
Methodology: Iterative, bootstrap, formal-methods-driven, AI-assisted, community-focused.
Bootstrap Language Recommendation: Rust (due to performance, safety, strong ecosystem, and good tooling for parsing, LLVM, and potentially formal methods integration). Adapt commands if using OCaml/Haskell.
Tracking: Mark steps as completed by changing [ ] to [v] upon successful implementation and verification/testing. Use Git commit messages referencing the Task ID (e.g., "feat(P0T1): Define core operational semantics v0.1"). Log significant design decisions and challenges encountered in a DESIGN_LOG.md file.

Phase 0: Foundational Formalism & Core ASG
Goal: Establish the mathematical and structural bedrock of Synapse. This phase focuses on defining the precise meaning of the core language fragment and creating the fundamental data structures (ASG) and basic tools (parser, formatter, CLI) needed to represent and manipulate code according to this definition. Formalism here prevents ambiguity later.

Task P0T1: Formal Semantics (Core) Definition
...
[v] Review Point: Have a peer (or use the AI agent's analytical capabilities) review the rules for internal consistency, completeness (covering all defined syntax), and correctness according to PL theory standards. Ensure the proof assistant formalization matches the document.

Task P0T2: ASG Schema Definition (v1)
...
[v] Review Point: Cross-reference every construct in the formal semantics with its representation in the ASG schema. Ensure all necessary connections (like variable definitions) can be represented. Check for potential ambiguities.

Task P0T3: Core ASG Libraries Implementation
...
[v] Review Point: Check API design, serialization correctness, hashing stability and strategy. Ensure tests provide good coverage.

Task P0T4: Minimal Parser & Pretty Printer Implementation
...
[v] Review Point: Check parser correctness and error reporting. Verify formatter output matches the defined syntax. Ensure round-trip tests pass.

Task P0T5: Basic CLI & Linter (Level 0) Implementation
Goal: Create the main user interface tool (synapse_cli) integrating the parser, formatter, and adding basic (Level 0) structural validation of the ASG.
Input: asg_core, parser_core, formatter_core libraries.
Output: Executable binary (target/debug/synapse_cli).
Instructions:
[v] Setup Project:
[v] cargo new synapse_cli
[v] Configure Dependencies (synapse_cli/Cargo.toml):
[v] Add asg_core, parser_core, formatter_core as path dependencies.
[v] Add clap (with derive feature) for CLI argument parsing.
[v] Add thiserror or anyhow for easier error handling in the application.
[v] Define CLI Interface (synapse_cli/src/main.rs):
[v] Use clap::Parser derive macro to define the main Cli struct and subcommands (Commands enum).
[v] Define subcommands:
Parse { input_file: std::path::PathBuf }
Format { input_file: std::path::PathBuf, #[clap(short, long)] output_file: Option<std::path::PathBuf> }
Lint { input_file: std::path::PathBuf }
DumpAsg { input_file: std::path::PathBuf, #[clap(long, arg_enum, default_value = "binary")] format: DumpFormat } (Define DumpFormat enum: Binary, Json).
[v] Implement Subcommand Logic:
[v] Parse: Call parser_core::parse_file(input_file), report success or display formatted error.
[v] Format: Parse file, call formatter_core::format_asg, print to stdout or save to output file. Handle errors.
[v] DumpAsg: Parse file, call asg_core::save_asg_binary or the JSON equivalent, write to stdout.
[v] Lint:
Parse file into an AsgGraph.
Implement the lint_graph(graph: &AsgGraph) -> Vec<LintError> function in a new synapse_cli/src/linter.rs module.
Level 0 Checks:
[v] Traverse the graph.
[v] Check for graph structural integrity (e.g., node IDs referenced actually exist in the graph map).
[v] Perform basic scope checking: Ensure TermVariable nodes link to a valid binder (e.g., TermLambda binder) within an enclosing scope. This requires building a symbol table during traversal.
[v] Check for simple type mismatches detectable without full inference (e.g., applying a non-lambda value, assigning to a non-ref).
Define LintError struct including error code (e.g., L001), message, relevant node_id, and potentially SourceLocation (requires mapping node IDs back to source - store location in ASG Metadata).
Report lint errors found.
[v] Implement Main Function: Parse CLI args using Cli::parse(), match on the subcommand, call the corresponding logic, handle errors gracefully (print user-friendly messages).
[v] Best Practice (TDD/Testing): Create integration tests (e.g., in synapse_cli/tests/cli_tests.rs) using test files:
[v] Test each subcommand with valid inputs.
[v] Test error handling for non-existent files, parse errors, lint errors.
[v] Test output formats (format, dump-asg).
[v] Best Practice (CI): Create a GitHub Actions workflow (.github/workflows/ci.yml):
[v] Trigger on push/pull_request.
[v] Setup Rust environment.
[v] Run cargo fmt --check --all.
[v] Run cargo clippy --all-targets -- -D warnings.
[v] Run cargo build --all-targets --release.
[v] Run cargo test --all-targets.
[v] Best Practice (Documentation): Create a README.md for synapse_cli explaining usage and commands.
[v] Review Point: Verify CLI commands work as expected. Check linter logic for correctness on basic cases. Confirm CI pipeline passes. Ensure user-facing errors are helpful.

(End of Phase 0)
...