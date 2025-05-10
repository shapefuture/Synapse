# Synapse Design Log

This document tracks significant design decisions made during the development of the Synapse language.

## Core Language Design

### Formalism Selection (2023-07-01)

**Decision**: We selected Small-step Structural Operational Semantics (SOS) as the primary semantic style for the core language.

**Rationale**: Small-step semantics offer several advantages for our use case:
- They provide a fine-grained view of program execution, making it easier to reason about effects and concurrent execution later.
- They naturally support reasoning about non-terminating computations.
- They align well with our plans for a step-based debugger and omniscient debugging.
- The transition relation `⟨t, σ⟩ → ⟨t', σ'⟩` provides a clear way to model state changes, which is essential for our imperative features.

**Alternatives Considered**:
- Big-step semantics: Simpler but harder to extend for concurrency and effects.
- Denotational semantics: More abstract but less intuitive for reasoning about operational behavior.
- Axiomatic semantics: Better for verification but less suitable as a definition of the language.

### ASG Structure (2023-07-05)

**Decision**: We're using a flat graph structure with explicit node IDs for the Abstract Semantic Graph (ASG) rather than a nested tree structure.

**Rationale**:
- Explicit node IDs allow for more efficient traversal and manipulation of the graph.
- A flat structure simplifies serialization/deserialization.
- It allows for sharing of subgraphs and avoids duplication.
- Enables bidirectional links (e.g., from variable uses to definitions).
- Makes it easier to annotate nodes with additional information (e.g., types, effects).

**Alternatives Considered**:
- Nested tree structure (traditional AST): Would make references between distant nodes more complex.
- Hybrid approach with nesting for simple expressions: Added complexity without significant benefits.

### Type System Foundation (2023-07-10)

**Decision**: The core type system is built on a simply-typed lambda calculus extended with reference types and a basic effect system.

**Rationale**:
- Provides a solid foundation with well-understood properties.
- Simple enough to implement and reason about, while still supporting key features.
- Can be extended incrementally with more advanced features (quantitative types, dependent types).
- Reference types give us the ability to model mutable state cleanly.
- Basic effect tracking sets the stage for more sophisticated effect systems later.

**Alternatives Considered**:
- Starting directly with dependent types: Too complex for the initial implementation.
- Pure functional approach without references: Would make interfacing with imperative code more difficult.
- Object-oriented foundation: Less amenable to formal verification.

## Implementation Decisions

### Bootstrap Language (2023-07-15)

**Decision**: We're using Rust as the implementation language for the initial compiler and runtime.

**Rationale**:
- Strong type system helps prevent bugs in the compiler itself.
- Memory safety without garbage collection is important for a compiler/runtime.
- Excellent FFI capabilities for interfacing with LLVM and other libraries.
- Good support for parsing (through libraries like LALRPOP) and binary protocols (Protobuf).
- Strong ecosystem for building CLI tools, servers, and other components.
- Performance characteristics align with our goals.

**Alternatives Considered**:
- OCaml: Common for compilers, great type system, but smaller ecosystem and less mainstream.
- Haskell: Excellent for language implementation, but steeper learning curve.
- C++: Powerful but lacks safety guarantees.

### Serialization Format (2023-07-20)

**Decision**: We're using Protocol Buffers for serializing the ASG and other structured data.

**Rationale**:
- Schema-based, allowing for backward/forward compatibility.
- Efficient binary format with good library support.
- Cross-language support will be helpful as we add more tools and services.
- Works well with the flat graph structure we've chosen for the ASG.

**Alternatives Considered**:
- JSON: More human-readable but less efficient and lacks schema validation.
- Cap'n Proto: Faster than Protobuf but less mature ecosystem.
- Custom binary format: Would require more development effort.

### Parser Technology (2023-07-25)

**Decision**: We're using LALRPOP as the parser generator for the initial implementation.

**Rationale**:
- Type-safe parser generator that integrates well with Rust.
- Generates LR parsers, which are efficient for our grammar.
- Good error reporting capabilities.
- Flexible enough to handle our syntax, including the planned extensions.
- Allows embedding Rust code directly in the grammar.

**Alternatives Considered**:
- Hand-written recursive descent: More work but more flexibility.
- Parser combinators (like `nom`): Less efficient for complex grammars.
- ANTLR: Powerful but less integrated with Rust.

## Future Design Considerations

### Quantitative Types Implementation

We're considering different approaches for implementing quantitative types:
- Explicit annotations for linearity/affinity vs. inference
- Integration with the borrow checker vs. a more separate system
- Handling of region-based memory management

### Dependent Types Strategy

For dependent types, we need to decide:
- How much of the type checking should be done at compile time vs. runtime
- Integration with SMT solvers for constraint solving
- Proof obligation representation and discharge mechanisms

### Effect System Design

Key decisions for the effect system include:
- Granularity of effects (coarse-grained vs. fine-grained)
- Effect polymorphism approach
- Handler implementation strategy (e.g., delimited continuations vs. monadic)