use crate::{expression::Expression, value::Value, vm::VM, Type};
use std::{collections::HashMap, rc::Rc};

type BuiltinMethod = fn(&mut VM, &Value, &[Value]) -> Value;

pub enum Method {
    Builtin(BuiltinMethod),
    Custom { body: Expression },
}

pub fn default_methods() -> HashMap<Type, HashMap<String, Rc<Method>>> {
    HashMap::from([(
        Type::String,
        HashMap::from([
            (
                "println".to_owned(),
                Rc::new(Method::Builtin(|_vm, this, _arguments| {
                    let Value::String(this) = this else { todo!() };
                    println!("{this}");
                    Value::Unit
                })),
            ),
            (
                "concat".to_owned(),
                Rc::new(Method::Builtin(|_vm, this, arguments| {
                    let Value::String(this) = this else { todo!() };
                    Value::String(
                        std::iter::once(&**this)
                            .chain(arguments.iter().map(|argument| {
                                match argument {
                                    Value::String(argument) => &**argument,
                                    _ => todo!(),
                                }
                            }))
                            .collect::<String>(),
                    )
                })),
            ),
        ]),
    )])
}
