//! Level 2 types: Add type variables and universal types for System F

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Bool,
    Function(Box<Type>, Box<Type>),
    Ref(Box<Type>),
    Var(u64), // Unification/type variables (both term and type level)
    ForAll(Vec<u64>, Box<Type>), // ∀ type variables. body
    ADT(String, Vec<Type>),      // Algebraic data type, parameterized
}