//! UPIR Attribute system.

#[derive(Debug, Clone)]
pub enum Attribute {
    String(String),
    Integer(i64),
    Bool(bool),
    Type(super::types::TypeId),
}