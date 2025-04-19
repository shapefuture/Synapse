//! Core type representations for Synapse Level 1 type checker.

/// Hindley-Milner types for the Synapse core language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Bool,
    Function(Box<Type>, Box<Type>),
    Ref(Box<Type>),
    Var(u64), // Unification variables
}

/// Type schemes allow let-polymorphism.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScheme {
    pub vars: Vec<u64>,
    pub body: Type,
}