use crate::vm::ClassID;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Object(ClassID),
    Unit,
    Bool,
    I32,
    String,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Object(class_id) => write!(f, "Class_{class_id}"),
            Self::Unit => f.write_str("Unit"),
            Self::Bool => f.write_str("Bool"),
            Self::I32 => f.write_str("I32"),
            Self::String => f.write_str("String"),
        }
    }
}
