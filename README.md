# Synapse Language — Core Compiler and Toolchain

## Overview

Synapse is a language and metaprogramming ecosystem designed with formal foundations, robust effect/type analysis, algebraic datatypes, parametric polymorphism, modern ADT/pattern-matching, and effect tracking throughout its compilation pipeline.

## Key Features

- **Formal Semantics**: Backed by Coq mechanization of type soundness (progress & preservation).
- **First-Class ASG**: Abstract Semantic Graph core, versioned for extensibility.
- **Advanced Type System**: Hindley-Milner and System F style polymorphism, ADTs, and full effect metadata on code.
- **Effect Tracking**: Enforced from parsing to IR to backend.
- **Extensible IR (UPIR)**: Universally extensible MLIR-inspired core, supports algebraic types, matches, effects.
- **LLVM/Native Backend**: End-to-end pipeline down to native code, with runtime.
- **Modern CLI**: Unified tool for parsing, checking, lowering, compiling, and effect checks.

## Quick Start

```sh
cargo build --workspace
# Type check and effect check
synapse_cli type-check-effects examples/adt_polymorphic.syn --allow-effect IO,Pure
# Lower to IR
synapse_cli lower-upir examples/adt_polymorphic.syn
# Compile to native
synapse_cli compile examples/adt_polymorphic.syn -o a.out
```

## Documentation

- [CLI Usage](docs/CLI.md)
- [Advanced Type/Eff/ADT rationale](docs/asg_schema_v2_rationale.md)
- [Effect System Design](docs/effect_system_rationale.md)
- [Core Semantics & Proofs](docs/semantics/core_v0.1_soundness.md, proofs/core_v0.1_soundness.v)

## Test and Contribute

- Run all tests: `cargo test --workspace`
- See integration tests for roundtrip examples and effect system regressions.
- Contributions via PRs and GitHub issues welcome as described in `CONTRIBUTING.md`.