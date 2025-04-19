//! UPIR type system.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u64);

#[derive(Debug, Clone)]
pub struct Type {
    pub id: TypeId,
    pub kind: TypeKind,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Builtin(BuiltinType),
    Ptr { element_type: TypeId },
    Struct { field_types: Vec<TypeId> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinType {
    I32, I64, F32, F64, Bool, Void,
}