# Synapse Programming Language

Vision: Computationally verifiable, AI-native, adaptive system for human-AI collaboration.

Synapse is a programming language designed with the following key principles:

- **Verifiability First**: Core semantics are formally defined and verified
- **AI-Native**: Designed for seamless human-AI collaboration
- **Adaptive**: Multiple projections (syntax forms) for the same semantic model
- **Resource-Aware**: Quantitative type system for precise resource management
- **Effect-Tracked**: Effect system for reasoning about computational effects

## Project Structure

The Synapse project is organized as a multi-crate Rust workspace:

### Core Libraries
- `asg_core`: Abstract Semantic Graph core library
- `parser_core`: Parser for the minimal text format
- `formatter_core`: Pretty printer for ASG
- `type_checker_l1`: Basic type checker (Hindley-Milner)
- `upir_core`: Universal Polymorphic Intermediate Representation
- `asg_to_upir`: Compiler stage to translate ASG to UPIR
- `upir_to_llvm`: Compiler stage to translate UPIR to LLVM IR
- `synapse_runtime`: Minimal runtime library

### Tools
- `synapse_cli`: Command-line interface
- `synapse_lsp`: Language Server Protocol implementation
- `synapse_ai_api`: AI integration API
- `synapse_debugger`: Debugging tools
- `synapse_pkg`: Package manager
- `synapse_collab_server`: Collaborative editing server

### Advanced Type Checkers
- `type_checker_l2`: Quantitative and effect types
- `type_checker_l3_core`: Dependent types with SMT solver integration
- `type_checker_l3_full`: Full dependent type system

### Additional Backends
- `upir_to_spirv`: UPIR to SPIR-V for GPU
- `upir_to_qsim`: UPIR to quantum simulator

### Other Components
- `proof_manager`: Proof management
- `proof_synthesis_assist`: Proof synthesis assistance
- `synapse_uart`: Universal Abstract Representation Translator
- `verified_ffi`: Verified Foreign Function Interface
- `ethics_checker`: Ethics checking
- `macro_expander`: Macro expansion
- `aspe_pythonic_v1`: Pythonic projection

## Development Status

This project is currently in early development, following the phased implementation plan outlined in `plan.md`.

See `DESIGN_LOG.md` for architecture decisions and `CONTRIBUTING.md` for contribution guidelines.

## Getting Started

Prerequisites:
- Rust (latest stable)
- LLVM development libraries
- Protocol Buffers compiler (protoc)

To build the project:
```bash
cargo build
```

To run the CLI:
```bash
cargo run -p synapse_cli
```
