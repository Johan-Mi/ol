use crate::vm::ClassID;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Object(ClassID),
    Unit,
    Bool,
    I32,
    String,
}
