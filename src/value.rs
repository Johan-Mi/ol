use crate::{object::Object, typ::Type};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Object(Rc<Object>),
    Unit,
    Bool(bool),
    I32(i32),
    String(String),
}

impl Value {
    pub fn typ(&self) -> Type {
        match self {
            Self::Object(object) => Type::Object(object.class),
            Self::Unit => Type::Unit,
            Self::Bool(_) => Type::Bool,
            Self::I32(_) => Type::I32,
            Self::String(_) => Type::String,
        }
    }
}
