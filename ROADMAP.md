# Synapse Project Roadmap

## Completed Milestones

- Phase 0: Formal semantics, ASG core/library, parser/formatter/cli, CI, minimal end-to-end pipeline. (v0.1.x)
- Phase 1: Core type inference (Hindley-Milner), UPIR core, end-to-end native/LLVM pipeline, runtime, soundness mechanization. (v0.1.x)
- **Phase 2 (v0.2.0): Major Type System Milestone**
  - System F polymorphism (universals, TypeAbs/TypeApp)
  - Algebraic datatypes, pattern matching, sum types
  - Effect system: node tags, static enforcement, full pipeline
  - CLI support and user-facing documentation
  - Integration and regression tests for all new language features

## Next Steps

- Feedback round from users/contributors; refine pipeline and CLI for next usability phase.
- Ecosystem: Begin work on standard libraries (effect-safe IO, container ADTs, etc).
- Phase 3:
  - Advanced optimization/verification pipeline (verified lowering, proofs in effect system, memory, etc).
  - AI code assistance and metaprogramming integration (per strategic plan).
  - Community package/collaboration tooling.
- Documentation, QA, public alpha/beta and onboarding.

## See Also

- [CHANGELOG.md](CHANGELOG.md)
- [plan.md](plan.md)