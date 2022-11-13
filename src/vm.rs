use crate::{
    expression::Expression, method::Method, object::Object, typ::Type,
    value::Value,
};
use std::{collections::HashMap, rc::Rc};

pub struct VM {
    methods: HashMap<Type, HashMap<String, Rc<Method>>>,
    local_variables: Vec<Value>,
    class_id_counter: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassID(usize);

impl VM {
    pub fn new() -> Self {
        Self {
            methods: HashMap::from([(
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
                                            Value::String(argument) => {
                                                &**argument
                                            }
                                            _ => todo!(),
                                        }
                                    }))
                                    .collect::<String>(),
                            )
                        })),
                    ),
                ]),
            )]),
            local_variables: Vec::new(),
            class_id_counter: 0,
        }
    }

    pub fn run(&mut self, main_type: ClassID) {
        let main_method = self
            .methods
            .get(&Type::Object(main_type))
            .unwrap()
            .get("main")
            .unwrap()
            .clone();
        let this = Value::Object(Rc::new(Object {
            class: main_type,
            properties: HashMap::default(),
        }));
        self.invoke_method(&main_method, &this, &[]);
    }

    pub fn new_class_id(&mut self) -> ClassID {
        self.class_id_counter += 1;
        ClassID(self.class_id_counter)
    }

    pub fn add_method(
        &mut self,
        this_type: Type,
        name: String,
        method: Rc<Method>,
    ) {
        self.methods
            .entry(this_type)
            .or_insert_with(Default::default)
            .insert(name, method);
    }

    fn invoke_method(
        &mut self,
        method: &Method,
        this: &Value,
        arguments: &[Value],
    ) -> Value {
        match method {
            Method::Builtin(f) => f(self, this, arguments),
            Method::Custom { body } => self.evaluate_expression(body),
        }
    }

    fn evaluate_expression(&mut self, expression: &Expression) -> Value {
        match expression {
            Expression::Literal(value) => value.clone(),
            Expression::MethodCall {
                name,
                this,
                arguments,
            } => {
                let this = self.evaluate_expression(this);
                let this_type = this.typ();
                let method = self
                    .methods
                    .get(&this_type)
                    .unwrap()
                    .get(name)
                    .unwrap()
                    .clone();
                let arguments = arguments
                    .iter()
                    .map(|argument| self.evaluate_expression(argument))
                    .collect::<Vec<_>>();
                self.invoke_method(&method, &this, &arguments)
            }
            Expression::LocalVariable {
                name_or_de_brujin_index: index,
            } => self
                .local_variables
                .get(self.local_variables.len() - 1 - *index as usize)
                .unwrap()
                .clone(),
            Expression::LetIn {
                name: (),
                bound,
                body,
            } => {
                let bound = self.evaluate_expression(bound);
                self.local_variables.push(bound);
                let result = self.evaluate_expression(body);
                self.local_variables.pop();
                result
            }
            Expression::IfThenElse {
                condition,
                if_true,
                if_false,
            } => {
                let Value::Bool(condition) = self.evaluate_expression(condition) else { todo!() };
                self.evaluate_expression(if condition {
                    if_true
                } else {
                    if_false
                })
            }
            Expression::Do(steps) => steps
                .iter()
                .map(|step| self.evaluate_expression(step))
                .last()
                .unwrap_or(Value::Unit),
        }
    }
}
