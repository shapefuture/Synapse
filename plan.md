# Synapse Implementation Plan (v3) - Detailed Steps

...(Phase 0, Phase 1 content unchanged/skipped)...

Phase 2: Enhanced Type Checking & Inference (Level 2)
Goal: Extend the type system and checker with advanced features such as parametric polymorphism (System F style), algebraic data types (ADTs), pattern matching, and effect system infrastructure. Propagate these extensions through all compiler and IR stages, with full user/CLI support and tests.

Task P2T1: Extension API and Schema for Polymorphism, ADTs, Effects
[v] Draft and implement `asg_schema_v2.proto` adding TypeAbs, TypeApp, DataDef, DataCtor, DataMatch, and effect tags with rationale in docs.
[v] Update asg_core and parser for new node types and effect metadata, round-trip tested.
[v] Document extension points and migration.

Task P2T2: Type Checker and Type Core Extensions—System F, ADTs, Effects
[v] Implement type context, inference, and checking for TypeAbs, TypeApp, DataDef, DataCtor, DataMatch
[v] Enforce effect tag propagation/compatibility, CLI effect check command.
[v] Update all unit and integration tests, document new type theory.

Task P2T3: UPIR & Lowering Pipeline for Polymorphism, ADTs, Effects
[v] Extend UPIR and builder with data type, match, polymorphic function support.
[v] Lower ASG System F, ADT, and effect nodes to UPIR with full IR mapping.
[v] Integration tests for polymorphic/ADT programs.

Task P2T4: UPIR-to-LLVM Lowering and Backend Update
[v] Propagate data types, match, effect metadata at IR/LLVM backend level (core.match stub for codegen, tag IR).
[v] Document LLVM IR representation and round-trip tests.

Task P2T5: CLI and E2E Pipeline Update
[v] Update CLI: type-check, effect-check, lower-upir, doc and tests for new features.
[v] Integration tests for polymorphic ADTs, effect tags; user documentation (CLI.md etc.)
[v] Mark milestone as v0.2.0 and prepare for next-phase or public release.

(End of Phase 2)
# Phase 3... (continues)