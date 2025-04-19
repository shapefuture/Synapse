//! Core type representations for Synapse Level 1 type checker.

/// Hindley-Milner types for the Synapse core language.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl Type {
    /// Create a fresh type variable (unification variable)
    pub fn fresh_var(counter: &mut u64) -> Self {
        let v = *counter;
        *counter += 1;
        Type::Var(v)
    }

    /// Apply a substitution to a type
    pub fn apply(&self, subst: &std::collections::HashMap<u64, Type>) -> Type {
        match self {
            Type::Var(id) => subst.get(id).cloned().unwrap_or(self.clone()),
            Type::Function(a, b) => {
                Type::Function(Box::new(a.apply(subst)), Box::new(b.apply(subst)))
            }
            Type::Ref(t) => Type::Ref(Box::new(t.apply(subst))),
            base => base.clone(),
        }
    }
}