# ASG Schema v2 Rationale — Polymorphism, ADTs, and Effects

## Goals

- Support for parametric polymorphism by modeling type abstraction/application explicitly in the ASG.
- Adding rich algebraic datatypes, with data definitions, constructors, and pattern matching.
- Effect tracking: record effect tags or capabilities on any node (for static or dynamic analysis).

## Main Additions

### 1. Type Abstraction/Application
- **TypeAbs** nodes represent universally quantified lambda abstractions at the type level (for parametric polymorphism, e.g., `forall T. ...`).
- **TypeApp** nodes represent applying a type abstraction to specific type argument nodes, supporting instantiation of generics.

### 2. Algebraic Datatypes (ADTs)
- **DataDef**, with constructor list and type parameters becomes a first-class ASG node.
- **DataCtor**: Constructor info for a datatype, can be referenced at construction.
- **DataMatch**: Supports destructuring and matching on a value of an ADT using arms for each constructor and binding pattern variables.

### 3. Effect Metadata
- **EffectMeta**: Node metadata (on any node) to attach a list of effect tags (e.g., "IO", "State", "Exception"), for effect system analysis and future effect polymorphism.

## Extension points

- All new node types include explicit child/argument node links for traversal.
- Type checker and runtime pass extension points documented in the type checker and lowering crates.

## Upgrading ASG_core and Type Checker

- Rust structs and type logic should mirror the new schema fields, type/tags, and reference node IDs.
- Ensure round-trip and pretty-print logic is robust to added node types, maintaining compatibility when v1-only nodes are present.

## Future

- This schema sets the stage for implementing full System F and GADTs, as well as extensible effect tracking both at type level and IR level.