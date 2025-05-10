# Changelog

## [v0.2.0] — Major Type System, ADT, and Effect Infrastructure Milestone

### Features & Core Changes

- Parametric polymorphism (System F): Add TypeAbs, TypeApp nodes and full type checker, UPIR, and CLI support for universally quantified generics.
- Algebraic data types (ADTs): Support data definitions, constructor, and pattern matching (DataDef, DataCtor, DataMatch) at ASG, IR, backend.
- Pattern matching: UPIR and backend (core.match op, MatchInfo), tests and pretty-printing.
- Effect system: Per-node effect tags (effect_meta), static effect checking/enforcement at type check and CLI, full pipeline propagation.
- CLI: New commands `type-check-effects` and `lower-upir` for ADT/polymorphism/effect features.
- Documentation: Update README, CLI.md, asg_schema_v2_rationale.md, effect_system_rationale.md for all new features.
- Integration tests: Cover polymorphism, ADT, and effect scenarios across CLI, pipeline, and backend.

### Status

All core Phase 2 features are complete, tested, and exposed at user/CLI and developer level.

### Next

Prepare for community alpha/beta, ecosystem/library expansion, and (if in plan) AI integration and formalized effect/memory models.