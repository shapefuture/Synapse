# Synapse CLI

Command-line interface for the Synapse programming language.

## Overview

The Synapse CLI provides a set of tools for working with Synapse source files, including parsing, formatting, linting, and ASG (Abstract Semantic Graph) manipulation.

## Installation

### Prerequisites

- Rust toolchain (stable)
- Protobuf compiler (protoc)

### Building

```bash
# From the root of the Synapse repository
cargo build -p synapse_cli --release

# The binary will be available at target/release/synapse_cli
```

## Usage

```
synapse_cli [SUBCOMMAND]
```

### Subcommands

#### Parse

Parse a Synapse source file and validate its structure.

```bash
synapse_cli parse <INPUT_FILE>
```

Example:
```bash
synapse_cli parse examples/function.syn
```

#### Format

Format a Synapse source file according to the language's style guidelines.

```bash
synapse_cli format <INPUT_FILE> [-o <OUTPUT_FILE>]
```

If no output file is specified, the formatted code is printed to stdout.

Example:
```bash
# Format and save to a new file
synapse_cli format examples/function.syn -o examples/function_formatted.syn

# Format and print to console
synapse_cli format examples/function.syn
```

#### Lint

Perform Level 0 (structural) linting on a Synapse source file.

```bash
synapse_cli lint <INPUT_FILE>
```

The linter checks for:
- Graph structural integrity (references to non-existent nodes)
- Variable scoping issues
- Simple type mismatches detectable without full type inference

Example:
```bash
synapse_cli lint examples/function.syn
```

#### Dump ASG

Dump the Abstract Semantic Graph (ASG) of a Synapse source file in binary or JSON format.

```bash
synapse_cli dump-asg <INPUT_FILE> [--format <FORMAT>] [-o <OUTPUT_FILE>]
```

Options:
- `--format`: Output format, either `binary` (default) or `json`
- `-o, --output-file`: Output file path (required for binary format)

Examples:
```bash
# Dump as binary
synapse_cli dump-asg examples/function.syn -o function.asg

# Dump as JSON to file
synapse_cli dump-asg examples/function.syn --format json -o function.json

# Dump as JSON to stdout
synapse_cli dump-asg examples/function.syn --format json
```

## Error Codes

### Lint Error Codes

- `L001`: Root node not found in graph
- `L002`: Variable references non-existent definition node
- `L003`: Lambda references non-existent binder node
- `L004`: Lambda references non-existent body node
- `L005`: Lambda references non-existent type annotation node
- `L006`: Application references non-existent function node
- `L007`: Application references non-existent argument node
- `L008`: Variable references definition node not in scope
- `L009`: Cannot apply arguments to a non-function value
- `L010`: Cannot dereference a non-reference value
- `L011`: Left-hand side of assignment must be a reference or variable

## Development

### Running Tests

```bash
# From the root of the Synapse repository
cargo test -p synapse_cli
```

### Adding New Commands

1. Add the command to the `Commands` enum in `main.rs`
2. Implement the command's logic in a separate function
3. Add the function call to the match expression in `main()`
4. Add tests for the new command in `cli_tests.rs`