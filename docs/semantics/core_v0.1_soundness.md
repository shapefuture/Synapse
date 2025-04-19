# Synapse Core v0.1 Type Soundness — Progress & Preservation

## Theorem Statements

**Preservation (Subject Reduction):**  
If Γ ⊢ e : T and e → e′ then Γ ⊢ e′ : T.

**Progress:**  
If ⊢ e : T then either e is a value, or there exists e′ such that e → e′.

## Proof Outline

- The proof is formalized in [`proofs/core_v0.1_soundness.v`](../proofs/core_v0.1_soundness.v) in Coq.
- Relies on the inductive definitions for terms, typing, and small-step operational semantics in the core language fragment.
- Proceeds by induction on the typing derivation (for preservation) and on structure of well-typed terms (for progress).
- Subcases for lambda abstraction, application, reference, dereference, and assignment are handled according to rules in the core calculus.

### Key Lemmas

- Substitution lemma
- Canonical forms
- Determinacy of step

## Status

The Coq files contain the main theorem statements and outline; detailed cases and sublemmas to be filled in as language features mature.

## See Also

- [`docs/semantics/core_v0.1.tex`](core_v0.1.tex)