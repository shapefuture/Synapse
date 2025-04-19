# Synapse Programming Language

Synapse is a next-generation programming language designed with a focus on formal verification, quantitative types, effect systems, and AI-assisted development.

## Overview

Synapse is built on the principle of "Verifiability First," aiming to provide a robust programming language that can express and verify complex properties while remaining practical and performant. Key features include:

- Formal semantics and type system
- Quantitative types for resource management
- Effect tracking
- Dependent types for expressing rich invariants
- AI integration for developer assistance
- Universal Adaptive Runtime (UART)
- Support for various hardware targets (CPU, GPU, Quantum)

## Project Status

Synapse is currently in the early development phase. We are working on implementing the core language features and compiler pipeline according to the phased approach outlined in our implementation plan.

### Completed Features

- [x] Formal semantics definition (core)
- [x] ASG schema definition
- [x] Core ASG libraries
- [x] Minimal parser and formatter
- [x] Basic CLI and linter (Level 0)

### Current Development Focus

- [ ] Type checker implementation (Level 1)
- [ ] Universal Polymorphic Intermediate Representation (UPIR)
- [ ] ASG-to-UPIR lowering
- [ ] UPIR-to-LLVM lowering
- [ ] Minimal runtime implementation

## Getting Started

### Prerequisites

- Rust toolchain (stable)
- Protobuf compiler (protoc)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/synapse-lang/synapse.git
cd synapse

# Build the project
cargo build --release

# Run tests
cargo test
```

### Using the CLI

```bash
# Parse a Synapse file
cargo run -- parse examples/hello.syn

# Format a Synapse file
cargo run -- format examples/hello.syn -o formatted.syn

# Lint a Synapse file
cargo run -- lint examples/hello.syn

# Dump the ASG to JSON
cargo run -- dump-asg examples/hello.syn --format json -o hello.json
```

## Language Example

```
# A simple function in Synapse
(x: Int) => x * x + 1

# Function with effects
(x: Int) => perform('IO', x + 1)

# Reference operations
(r: Ref Int) => {
  !r := !r + 1;
  !r
}
```

## Project Structure

- `asg_core/` - Core library for the Abstract Semantic Graph
- `parser_core/` - Parser for the core language
- `formatter_core/` - Pretty printer for the core language
- `synapse_cli/` - Command-line interface
- `docs/` - Documentation, including formal specifications
- `schemas/` - Protocol Buffer schemas

## Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

## License

This project is licensed under the [MIT License](LICENSE).

## Acknowledgments

Synapse draws inspiration from many languages and research projects in the programming language community, including but not limited to Rust, OCaml, Lean, F*, and Linear Haskell.