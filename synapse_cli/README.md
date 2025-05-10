# synapse_cli

Command-line interface for the Synapse language:
- Parsing, formatting, linting, and introspecting Synapse source files (ASG-based).

## Features

- **parse**: Parse Synapse minimal source and check for syntax errors.
- **format**: Pretty-print Synapse code in canonical minimal syntax.
- **lint**: Lint for structural errors (ASG well-formedness, scope, basic type anomalies).
- **dump-asg**: Output the parsed ASG in binary or JSON format for inspection/tooling.

## Installation

Recommended workflow: build in a Rust workspace with all foundational Synapse crates (see root README).
```
cargo build -p synapse_cli
```

## Usage

```
synapse_cli <COMMAND> [OPTIONS]
```

### Subcommands

#### `parse INPUT_FILE`

Parse the provided Synapse source file. Reports `Parse successful` or prints an error.

Example:
```
synapse_cli parse examples/add.syn
```

#### `format INPUT_FILE [-o OUTPUT_FILE]`

Parse and pretty-print (normalize) the input file. Output to stdout by default, or to specified file.

Example:
```
synapse_cli format examples/add.syn
```

#### `lint INPUT_FILE`

Runs Level 0 structural linting on the input file:
- Confirms all referenced node IDs exist in ASG.
- Checks TermVariable nodes point to valid definitions in scope.
- Warns about obvious application/assignment errors (e.g., applying a non-lambda).

Returns:
- Success if no errors, or prints them with error code, message, and node/location.

Example:
```
synapse_cli lint examples/add.syn
```

#### `dump-asg INPUT_FILE [--format {binary,json}]`

Parses and prints the underlying ASG as binary (protobuf) or pretty JSON.

Example:
```
synapse_cli dump-asg examples/add.syn --format json
```

## Lint Error Codes

- `L001`: Reference integrity error (node/edge points to missing node)
- `L002`: Scope error (TermVariable points to non-existent/invalid binder)
- `L003`: Application error (application of non-lambda value)
- `L004`: Assignment error (assigning to a non-reference)
- _Additional codes to be added as linter evolves_

Each error includes:
- Error code
- Explanation message
- Offending node ID
- Source location (if available)

## Error Handling

All commands use robust error reporting: parse/format/lint failures print user-friendly messages. Linter errors are non-fatal but yield exit code 2.

For development details, see `src/main.rs` and `src/linter.rs`.

## Development & Testing

### Running tests

```bash
cargo test -p synapse_cli
```

Tests include CLI end-to-end cases for parsing, formatting, linting, and dump-asg with valid/invalid inputs.

### Adding tests

- See `tests/cli_tests.rs` (integration tests).
- Add `.syn` source files in a `tests/data/` directory to exercise linter/format tests.

### Dependencies

- `clap` (CLI arg parsing)
- `thiserror` & `anyhow` (error handling)
- `asg_core`, `parser_core`, `formatter_core` (local crates)
- `serde_json` (ASG JSON dump)

## Roadmap

- Advanced linter checks, type inference, compilation pipeline (future subcommands)
- See [plan.md](../plan.md) for overall language/compiler development plan

## See Also

- [asg_core](../asg_core/)
- [parser_core](../parser_core/)
- [formatter_core](../formatter_core/)

---