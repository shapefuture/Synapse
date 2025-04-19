//! Abstract Syntax Tree for Synapse Core.
//!
//! This module defines an intermediate AST used during parsing
//! before conversion to the full ASG.

use std::fmt;

/// The top-level AST node, representing a complete Synapse source file.
#[derive(Debug, Clone)]
pub struct Root {
    pub expr: Expr,
}

/// Expression types in the AST.
#[derive(Debug, Clone)]
pub enum Expr {
    /// Variable reference: x
    Variable(String),
    
    /// Lambda abstraction: (x: ty) => expr
    Lambda(String, Option<Type>, Box<Expr>),
    
    /// Function application: f(arg)
    Application(Box<Expr>, Box<Expr>),
    
    /// Integer literal: 42
    IntLiteral(i64),
    
    /// Boolean literal: true, false
    BoolLiteral(bool),
    
    /// Primitive operation: op(args...)
    PrimitiveOp(String, Vec<Expr>),
    
    /// Reference creation: ref expr
    Ref(Box<Expr>),
    
    /// Dereference: !expr
    Deref(Box<Expr>),
    
    /// Assignment: expr1 := expr2
    Assign(Box<Expr>, Box<Expr>),
    
    /// Effect performance: perform(effect_name, expr)
    Perform(String, Box<Expr>),
}

/// Type expressions in the AST.
#[derive(Debug, Clone)]
pub enum Type {
    /// Int type
    Int,
    
    /// Bool type
    Bool,
    
    /// Function type: T1 -> T2
    Function(Box<Type>, Box<Type>),
    
    /// Reference type: Ref T
    Ref(Box<Type>),
    
    /// Unit type
    Unit,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Bool => write!(f, "Bool"),
            Type::Function(t1, t2) => write!(f, "({} -> {})", t1, t2),
            Type::Ref(t) => write!(f, "Ref {}", t),
            Type::Unit => write!(f, "Unit"),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Variable(name) => write!(f, "{}", name),
            Expr::Lambda(param, ty, body) => {
                if let Some(t) = ty {
                    write!(f, "({}: {}) => {}", param, t, body)
                } else {
                    write!(f, "({}) => {}", param, body)
                }
            }
            Expr::Application(func, arg) => write!(f, "{}({})", func, arg),
            Expr::IntLiteral(n) => write!(f, "{}", n),
            Expr::BoolLiteral(b) => write!(f, "{}", b),
            Expr::PrimitiveOp(op, args) => {
                write!(f, "{}(", op)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Expr::Ref(e) => write!(f, "ref {}", e),
            Expr::Deref(e) => write!(f, "!{}", e),
            Expr::Assign(e1, e2) => write!(f, "{} := {}", e1, e2),
            Expr::Perform(effect, e) => write!(f, "perform('{}', {})", effect, e),
        }
    }
}