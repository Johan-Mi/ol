use crate::{expression::Expression, value::Value, vm::VM};

type BuiltinMethod = fn(&mut VM, &Value, &[Value]) -> Value;

pub enum Method {
    Builtin(BuiltinMethod),
    Custom { body: Expression },
}
