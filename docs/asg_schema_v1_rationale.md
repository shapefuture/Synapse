# ASG Schema v1 Design Rationale

This document explains the design decisions behind the Abstract Semantic Graph (ASG) schema for the Synapse language.

## Overview

The ASG is the core data structure representing Synapse code in memory and on disk, serving as the foundation for analysis, transformation, and code generation throughout the compiler pipeline. It is designed to be a rich, complete representation of program semantics rather than just syntax.

## Key Design Decisions

### Flat Structure with Explicit Node IDs

We chose a flat structure for the ASG where all nodes are stored in a single list within the `AsgGraph` message, with explicit IDs representing edges between nodes. This approach was selected for several reasons:

1. **Queryability**: A flat structure makes it easier to locate nodes by ID without traversing a complex nested structure.
2. **Mutability**: Graph transformations become more manageable when nodes can be accessed directly rather than through nested paths.
3. **Referential Integrity**: Using explicit node IDs for references allows for bidirectional links and shared subtrees without duplicating nodes.
4. **Serialization Efficiency**: A flat structure simplifies serialization/deserialization logic and can be more efficient for large graphs.

### Comprehensive Node Type System

The ASG schema includes a diverse set of node types to represent all constructs in the formal semantics:

1. **Expressions & Terms**: Variables, lambdas, applications, literals, etc.
2. **Types**: Int, Bool, Function, Ref, etc.
3. **Effects**: Represented through EffectPerform.
4. **Metadata**: Source locations, annotations, etc.

This comprehensive type system ensures the ASG can accurately represent all semantic elements of Synapse code.

### Separation of Content from Structure

Each node has a `type` field and a `content` oneof field. This separation:

1. Allows for quick type checking without deserializing the entire content.
2. Provides a clear extensibility path for adding new node types.
3. Mirrors a traditional algebraic data type approach.

### Explicit Links for Variable Definitions

The `TermVariable` message includes a `definition_node_id` field, explicitly linking variable uses to their definitions. This design:

1. Eliminates the need for scope-based name resolution after parsing.
2. Makes variable binding and usage explicit in the graph structure.
3. Simplifies analyses like renaming, use-def chains, and type checking.

### Type Annotations as Optional Links

Type annotations are represented as optional links to `TypeNode` entities. This approach:

1. Allows for gradual typing or partial type information.
2. Separates type information from term structure, permitting independent transformation.
3. Enables easy addition of inferred types during type checking phases.

### Direct Representation of Semantic Constructs

The ASG directly represents semantic constructs like references, dereferences, and assignments rather than encoding them as function calls or compound expressions. This choice:

1. Maintains a close correspondence with the formal semantics.
2. Makes static analysis of memory operations more straightforward.
3. Simplifies verification and proof generation for memory safety.

### ProofObligation Nodes

Including explicit `ProofObligation` nodes in the ASG prepares the groundwork for formal verification:

1. Proof obligations can be directly associated with relevant code.
2. The status of each obligation can be tracked and updated during verification.
3. This design anticipates the integration of SMT solvers and proof assistants in later phases.

## Alternatives Considered

### Nested Tree Structure

A traditional AST-like nested structure was considered but rejected because:
- It would complicate handling cross-references and shared subtrees.
- Transformation and mutation would require complex traversal and path tracking.
- It would be less amenable to parallel processing and incremental updates.

### String-Based Variable References

Using string names for variable references was considered but rejected in favor of explicit node ID links:
- Name-based lookup would require maintaining a symbol table alongside the ASG.
- Renaming would be more error-prone and computationally expensive.
- Explicit links make the binding structure immediately apparent.

### Encoding Effects as Types

Representing effects within the type system (effect-decorated function types) was considered but deferred to a later phase:
- The current approach with explicit `EffectPerform` nodes is simpler for the initial implementation.
- It matches the operational semantics more directly.
- A more sophisticated effect system can be incorporated in later schema versions.

## Future Extensions

The ASG schema is designed with future expansion in mind:

1. **Quantitative Types**: The type system can be extended to include linear/affine types.
2. **Effect System**: More sophisticated effect tracking can be added.
3. **Dependent Types**: The type system can be enhanced to represent value dependencies.
4. **Module System**: New node types can be added to represent module boundaries and imports.

Each of these extensions will build upon the solid foundation established by the current schema design.