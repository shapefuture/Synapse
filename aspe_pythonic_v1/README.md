# ASPE: AI Syntax Projection Engine (Pythonic v1 Prototype)

- Translates between Pythonic code and Synapse canonical ASG (S-expression or JSON).
- Uses a transformer (LLM, e.g., T5/BART) for bidirectional code/ASG translation.
- Basic error-correction and explainable error pathway for parse failures.

## Usage

(Prototype/training loop, to be implemented. See plan.md Phase 3.3)

- `project_to_asg.py` — reads Pythonic code, outputs canonical ASG (via ML + parser core FFI)
- `project_to_pythonic.py` — reads ASG S-expr, outputs readable Pythonic code.

## Status

- Placeholder. Data, Python script, and demo model in progress.