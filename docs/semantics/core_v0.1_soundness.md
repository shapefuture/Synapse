# Synapse Core v0.1 Type Soundness — Progress & Preservation

## Theorem Statements

**Preservation (Subject Reduction):**  
If Γ ⊢ e : T and e → e′ then Γ ⊢ e′ : T.

**Progress:**  
If ⊢ e : T then either e is a value, or there exists e′ such that e → e′.

## Fully Worked Proofs

These theorems—with all supporting lemmas—are formalized in [`proofs/core_v0.1_soundness.v`](../proofs/core_v0.1_soundness.v) (Coq). The proofs are complete for pure STLC and sketched (see Admitted) for the extension to references, requiring generalization to store-typing and evaluation contexts.

### Key Lemmas

- Substitution lemma (proof complete)
- Canonical forms (proof complete)
- Determinacy of step (sketch provided)
- Additional cases for references and store left as exercise/future work.

## Mechanized Version

See `proofs/core_v0.1_soundness.v` for the Coq formalization and notes. Complete all `admit` for full mechanization.

## Status

- The standard cases (lambda calculus) are proven.
- Reference operations (creation, dereference, assignment) are sketched, pending full store semantics in the formalization.
- Foundation for verified core pipeline is established.

## See Also

- [`docs/semantics/core_v0.1.tex`](core_v0.1.tex)