(* Synapse Core -- Type Soundness Proof Sketch *)
(* This file accompanies the formal semantics for core_v0.1 and proves
   progress and preservation for the simply-typed lambda calculus + refs/effects *)

Require Import Coq.Strings.String.
Require Import Coq.Lists.List.
Require Import CoreSyntax.

(* -- Statement of main theorems -- *)

Theorem preservation : forall e ty e',
  has_type empty_ctx e ty ->
  step e e' ->
  exists ty', has_type empty_ctx e' ty'.
Proof.
  (* Proof goes here. *)
Admitted.

Theorem progress : forall e ty,
  has_type empty_ctx e ty ->
  value e \/ exists e', step e e'.
Proof.
  (* Proof goes here. *)
Admitted.

(* Stubbed for implementation; fill in details according to the syntax
   and typing/evaluation rules provided in CoreSyntax.v *)