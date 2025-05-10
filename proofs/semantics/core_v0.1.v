(* Synapse Core Language Formal Semantics (v0.1) *)
(* This file contains a formalization of the core Synapse language in Coq *)

Require Import Coq.Strings.String.
Require Import Coq.Lists.List.
Require Import Coq.Arith.Arith.
Require Import Coq.Bool.Bool.
Import ListNotations.

(* We define identifiers as strings *)
Definition id := string.

(* Types *)
Inductive ty : Type :=
  | TInt  : ty                (* Integer type *)
  | TBool : ty                (* Boolean type *)
  | TFun  : ty -> ty -> ty    (* Function type: τ₁ → τ₂ *)
  | TRef  : ty -> ty          (* Reference type: Ref τ *)
  | TUnit : ty.               (* Unit type for assignment *)

(* Effect labels *)
Inductive effect : Type :=
  | EName : string -> effect. (* Named effect *)

(* Terms *)
Inductive tm : Type :=
  | tvar  : id -> tm                        (* Variable *)
  | tabs  : id -> ty -> tm -> tm            (* Abstraction: λx:τ. t *)
  | tapp  : tm -> tm -> tm                  (* Application: t₁ t₂ *)
  | tint  : nat -> tm                       (* Integer constant *)
  | tbool : bool -> tm                      (* Boolean constant *)
  | tadd  : tm -> tm -> tm                  (* Addition: add(t₁, t₂) *)
  | tref  : tm -> tm                        (* Reference creation: ref t *)
  | tderef : tm -> tm                       (* Dereference: !t *)
  | tassign : tm -> tm -> tm                (* Assignment: t₁ := t₂ *)
  | tperform : effect -> tm -> tm           (* Effect performance: perform E t *)
  | tunit : tm.                             (* Unit value *)

(* Values *)
Inductive value : tm -> Prop :=
  | v_abs  : forall x T t, value (tabs x T t)
  | v_int  : forall n, value (tint n)
  | v_bool : forall b, value (tbool b)
  | v_unit : value tunit.

(* Type context: maps variables to their types *)
Definition context := list (id * ty).

(* Store locations *)
Definition loc := nat.

(* Store: maps locations to terms *)
Definition store := list (loc * tm).

(* The next available location in a store *)
Fixpoint next_loc (s : store) : loc :=
  match s with
  | nil => 0
  | (l, _) :: s' => S (max l (next_loc s'))
  end.

(* Typing judgment: Γ ⊢ t : τ *)
Reserved Notation "Gamma '⊢' t ':' T" (at level 40).

Inductive has_type : context -> tm -> ty -> Prop :=
  (* T-Var *)
  | T_Var : forall Gamma x T,
      In (x, T) Gamma ->
      Gamma ⊢ tvar x : T
  
  (* T-Abs *)
  | T_Abs : forall Gamma x T1 t T2,
      (x, T1) :: Gamma ⊢ t : T2 ->
      Gamma ⊢ tabs x T1 t : TFun T1 T2
  
  (* T-App *)
  | T_App : forall Gamma t1 t2 T1 T2,
      Gamma ⊢ t1 : TFun T1 T2 ->
      Gamma ⊢ t2 : T1 ->
      Gamma ⊢ tapp t1 t2 : T2
  
  (* T-Int *)
  | T_Int : forall Gamma n,
      Gamma ⊢ tint n : TInt
  
  (* T-Bool *)
  | T_Bool : forall Gamma b,
      Gamma ⊢ tbool b : TBool
  
  (* T-Add *)
  | T_Add : forall Gamma t1 t2,
      Gamma ⊢ t1 : TInt ->
      Gamma ⊢ t2 : TInt ->
      Gamma ⊢ tadd t1 t2 : TInt
  
  (* T-Ref *)
  | T_Ref : forall Gamma t T,
      Gamma ⊢ t : T ->
      Gamma ⊢ tref t : TRef T
  
  (* T-Deref *)
  | T_Deref : forall Gamma t T,
      Gamma ⊢ t : TRef T ->
      Gamma ⊢ tderef t : T
  
  (* T-Assign *)
  | T_Assign : forall Gamma t1 t2 T,
      Gamma ⊢ t1 : TRef T ->
      Gamma ⊢ t2 : T ->
      Gamma ⊢ tassign t1 t2 : TUnit
  
  (* T-Perform *)
  | T_Perform : forall Gamma e t T,
      Gamma ⊢ t : T ->
      Gamma ⊢ tperform e t : T
  
  (* T-Unit *)
  | T_Unit : forall Gamma,
      Gamma ⊢ tunit : TUnit

where "Gamma '⊢' t ':' T" := (has_type Gamma t T).

(* Substitution: [x ↦ s]t *)
Fixpoint subst (x : id) (s : tm) (t : tm) : tm :=
  match t with
  | tvar y      => if String.eqb x y then s else tvar y
  | tabs y T t1 => if String.eqb x y 
                   then tabs y T t1
                   else tabs y T (subst x s t1)
  | tapp t1 t2  => tapp (subst x s t1) (subst x s t2)
  | tint n      => tint n
  | tbool b     => tbool b
  | tadd t1 t2  => tadd (subst x s t1) (subst x s t2)
  | tref t1     => tref (subst x s t1)
  | tderef t1   => tderef (subst x s t1)
  | tassign t1 t2 => tassign (subst x s t1) (subst x s t2)
  | tperform e t1 => tperform e (subst x s t1)
  | tunit       => tunit
  end.

(* Location substitution: replaces a location expression with the actual location *)
Fixpoint loc_subst (x : id) (l : loc) (t : tm) : tm :=
  match t with
  | tvar y      => if String.eqb x y then tint l else tvar y (* Using tint for locations *)
  | tabs y T t1 => if String.eqb x y 
                   then tabs y T t1
                   else tabs y T (loc_subst x l t1)
  | tapp t1 t2  => tapp (loc_subst x l t1) (loc_subst x l t2)
  | tint n      => tint n
  | tbool b     => tbool b
  | tadd t1 t2  => tadd (loc_subst x l t1) (loc_subst x l t2)
  | tref t1     => tref (loc_subst x l t1)
  | tderef t1   => tderef (loc_subst x l t1)
  | tassign t1 t2 => tassign (loc_subst x l t1) (loc_subst x l t2)
  | tperform e t1 => tperform e (loc_subst x l t1)
  | tunit       => tunit
  end.

(* Operational semantics: ⟨t, σ⟩ → ⟨t', σ'⟩ *)
Reserved Notation "⟨ t , st ⟩ '-->' ⟨ t' , st' ⟩" (at level 40).

Inductive step : tm * store -> tm * store -> Prop :=
  (* E-App *)
  | ST_App : forall x T t v st,
      value v ->
      ⟨ tapp (tabs x T t) v, st ⟩ --> ⟨ subst x v t, st ⟩
      
  (* E-Add *)
  | ST_Add : forall n1 n2 st,
      ⟨ tadd (tint n1) (tint n2), st ⟩ --> ⟨ tint (n1 + n2), st ⟩
      
  (* E-Ref *)
  | ST_Ref : forall v st,
      value v ->
      let l := next_loc st in
      ⟨ tref v, st ⟩ --> ⟨ tint l, (l, v) :: st ⟩
      
  (* E-Deref *)
  | ST_Deref : forall l v st,
      In (l, v) st ->
      value v ->
      ⟨ tderef (tint l), st ⟩ --> ⟨ v, st ⟩
      
  (* E-Assign *)
  | ST_Assign : forall l v st st',
      value v ->
      (* Replace the value at location l in the store *)
      (forall l' v', In (l', v') st' <-> 
         (l' = l /\ v' = v) \/ (l' <> l /\ In (l', v') st)) ->
      ⟨ tassign (tint l) v, st ⟩ --> ⟨ tunit, st' ⟩
      
  (* E-Perform *)
  | ST_Perform : forall e v st,
      value v ->
      ⟨ tperform e v, st ⟩ --> ⟨ v, st ⟩
      
  (* Context rules *)
  
  (* E-App1 *)
  | ST_App1 : forall t1 t1' t2 st st',
      ⟨ t1, st ⟩ --> ⟨ t1', st' ⟩ ->
      ⟨ tapp t1 t2, st ⟩ --> ⟨ tapp t1' t2, st' ⟩
      
  (* E-App2 *)
  | ST_App2 : forall v t2 t2' st st',
      value v ->
      ⟨ t2, st ⟩ --> ⟨ t2', st' ⟩ ->
      ⟨ tapp v t2, st ⟩ --> ⟨ tapp v t2', st' ⟩
      
  (* E-Add1 *)
  | ST_Add1 : forall t1 t1' t2 st st',
      ⟨ t1, st ⟩ --> ⟨ t1', st' ⟩ ->
      ⟨ tadd t1 t2, st ⟩ --> ⟨ tadd t1' t2, st' ⟩
      
  (* E-Add2 *)
  | ST_Add2 : forall v t2 t2' st st',
      value v ->
      ⟨ t2, st ⟩ --> ⟨ t2', st' ⟩ ->
      ⟨ tadd v t2, st ⟩ --> ⟨ tadd v t2', st' ⟩
      
  (* E-Ref1 *)
  | ST_Ref1 : forall t t' st st',
      ⟨ t, st ⟩ --> ⟨ t', st' ⟩ ->
      ⟨ tref t, st ⟩ --> ⟨ tref t', st' ⟩
      
  (* E-Deref1 *)
  | ST_Deref1 : forall t t' st st',
      ⟨ t, st ⟩ --> ⟨ t', st' ⟩ ->
      ⟨ tderef t, st ⟩ --> ⟨ tderef t', st' ⟩
      
  (* E-Assign1 *)
  | ST_Assign1 : forall t1 t1' t2 st st',
      ⟨ t1, st ⟩ --> ⟨ t1', st' ⟩ ->
      ⟨ tassign t1 t2, st ⟩ --> ⟨ tassign t1' t2, st' ⟩
      
  (* E-Assign2 *)
  | ST_Assign2 : forall v t2 t2' st st',
      value v ->
      ⟨ t2, st ⟩ --> ⟨ t2', st' ⟩ ->
      ⟨ tassign v t2, st ⟩ --> ⟨ tassign v t2', st' ⟩
      
  (* E-Perform1 *)
  | ST_Perform1 : forall e t t' st st',
      ⟨ t, st ⟩ --> ⟨ t', st' ⟩ ->
      ⟨ tperform e t, st ⟩ --> ⟨ tperform e t', st' ⟩

where "⟨ t , st ⟩ '-->' ⟨ t' , st' ⟩" := (step (t, st) (t', st')).

(* Multi-step evaluation *)
Inductive multi_step : tm * store -> tm * store -> Prop :=
  | multi_refl : forall t st, multi_step (t, st) (t, st)
  | multi_step : forall t t' t'' st st' st'',
      ⟨ t, st ⟩ --> ⟨ t', st' ⟩ ->
      multi_step (t', st') (t'', st'') ->
      multi_step (t, st) (t'', st'').

Notation "⟨ t , st ⟩ '-->*' ⟨ t' , st' ⟩" := (multi_step (t, st) (t', st')).

(* Type soundness statements *)

(* Preservation: If Γ ⊢ t : τ and ⟨t, σ⟩ → ⟨t', σ'⟩, then Γ ⊢ t' : τ. *)
(* Note: This requires defining well-typed stores, which we'll need to formalize *)

(* Progress: If Γ ⊢ t : τ, then either t is a value or there exists t', σ' such that ⟨t, σ⟩ → ⟨t', σ'⟩. *)
(* Note: This also requires well-typed stores *)

(* We're stating the theorems but leaving the complete proofs for future implementation *)
(* These proofs require auxiliary lemmas about store typing that we'll develop *)

(* In practice, we would continue by defining store typing and proving these theorems *)