# Synapse CLI Usage and Advanced Type System Features

## Commands

- `compile` — Compile a Synapse source file to an executable.
- `type-check-effects` — Type check and check for effect violations.  
  Pass `--allow-effect IO,State` to whitelist enabled effect tags.
- `lower-upir` — Lower a source (or ASG) file to UPIR and pretty-print.

## Effect Checking

When using `type-check-effects`, all nodes with effect tags not explicitly allowed will cause a type check error. This lets you guarantee, for example, that only pure code can run in pure contexts.

## Polymorphism and ADTs

- The type checker and IR pipeline fully supports System F style polymorphic functions and algebraic data types (ADTs). Match and sum types are preserved to the backend and IR output.

## Example

```sh
synapse_cli type-check-effects example.syn --allow-effect IO,Pure
```