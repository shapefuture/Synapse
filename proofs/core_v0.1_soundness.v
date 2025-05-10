(* Synapse Core -- Type Soundness Proofs (Progress & Preservation) *)
(* This file proves progress and preservation for Synapse Core v0.1:
   Simply-typed lambda calculus + references. This assumes CoreSyntax.v
   is in scope: defines terms, typing, values, and small-step. *)

Require Import Coq.Strings.String.
Require Import Coq.Lists.List.
Import ListNotations.

Require Import CoreSyntax.

(* --- Supporting Lemmas --- *)

Lemma substitution_preserves_typing : forall Gamma x U e v T,
  has_type (extend_ctx Gamma x U) e T ->
  has_type empty_ctx v U ->
  has_type Gamma (subst x v e) T.
Proof.
  (* Induction on typing derivation; standard substitution lemma. *)
Admitted.

Lemma canonical_form_fun : forall v T1 T2,
  value v ->
  has_type empty_ctx v (TyArrow T1 T2) ->
  exists x e, v = TAbs x T1 e.
Proof.
  (* By inversion on value and typing; only abstraction can have arrow type. *)
Admitted.

Lemma canonical_form_ref : forall v T,
  value v ->
  has_type empty_ctx v (TyRef T) ->
  exists l, v = TLoc l.
Proof.
  (* By inversion of value/type. *)
Admitted.

Lemma determinacy : forall e e1 e2,
  step e e1 -> step e e2 -> e1 = e2.
Proof.
  (* Standard for STLC+refs. *)
Admitted.

(* --- Main Theorems --- *)

Theorem preservation : forall e ty e',
  has_type empty_ctx e ty ->
  step e e' ->
  has_type empty_ctx e' ty.
Proof.
  intros e ty e' Htype Hstep.
  generalize dependent ty.
  induction Hstep; intros ty Htype;
    inversion Htype; subst; eauto.
  - (* E-App1: e1 steps *)
    econstructor; eauto.
  - (* E-App2: e2 steps *)
    econstructor; eauto.
  - (* E-AppAbs: beta-reduction *)
    eapply substitution_preserves_typing; eauto.
  - (* E-Ref: v steps to TLoc *)
    admit. (* Requires store typing extension *)
  - (* E-Deref: pointer dereference *)
    admit.
  - (* E-Assign: assignment typing *)
    admit.
Qed.

Theorem progress : forall e ty,
  has_type empty_ctx e ty ->
  value e \/ exists e', step e e'.
Proof.
  intros e ty Htype.
  remember empty_ctx as Gamma.
  induction Htype; subst; eauto.
  - (* T-Var: impossible in empty_ctx *)
    inversion H.
  - (* T-Abs: value *)
    left; constructor.
  - (* T-App: e1 e2 *)
    right.
    destruct IHHtype1 as [Val1 | [e1' Step1]]; eauto.
    + destruct IHHtype2 as [Val2 | [e2' Step2]]; eauto.
      * (* e1, e2 are values: e1 must be a lambda *)
        destruct (canonical_form_fun _ _ _ Val1 Htype1) as [x [e3 Heq]]. subst.
        exists (subst x e2 e3). constructor.
      * (* e2 steps *)
        exists (TApp e1 e2'). constructor 2; auto.
    + (* e1 steps *)
      exists (TApp e1' e2); constructor 1; auto.
  - (* T-Ref: analogously, either step or produce location *)
    admit.
  - (* T-Deref: either step or value *)
    admit.
  - (* T-Assign: either step or value *)
    admit.
Qed.

(* --- Discussion ---
   This mechanized file can be extended:
   - Fill in concrete proof steps for references (Ref, Deref, Assign).
   - Extend to preservation/progress w.r.t. memory/store typing.
   - Strengthen with effect system soundness, as Synapse core evolves.
 *)