# Synapse Effect System — Rationale & Initial Semantics

## Goals

- Statical effect checking: ensure that code with effects (IO, State, Exception, etc.) is only invocable or usable within appropriate effect contexts.
- Support effect-metadata on all nodes; propagate and check these tags during type checking/lowering.
- Permit future effect polymorphism (polymorphic over sets of effects).
- Enable both simple capability-based and more refined effect system extensions later (e.g., algebraic effects, tracked usage).

## Current Implementation

- `effect_meta` is an optional field on every ASG node; set to a set of effect capability tags.
- ASG → UPIR lowering and all intermediate representations preserve effect tags.
- The type checker entrypoint `check_and_annotate_graph_v2_with_effects_check` will reject code that requires effects not available in the enclosing context.
- On error, effect violations are reported and compilation fails.

## Example

- A `pure` function cannot use any operation with effect tag IO.
- A function marked as requiring State or IO may only be called/transitively invoked in a context where those effects are permitted.

## Future

- Effect polymorphism: allow functions to abstract over effects, or quantify capabilities.
- Attach effect tags to additional constructs at IR, LLVM, or runtime as the pipeline matures.