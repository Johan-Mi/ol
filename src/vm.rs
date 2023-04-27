use crate::{
    expression::Expression,
    method::{default_methods, Method},
    object::Object,
    program::Program,
    resolve::Resolver,
    typ::Type,
    value::Value,
};
use anyhow::{Context, Result};
use std::{collections::HashMap, fmt, rc::Rc};

pub struct VM {
    methods: HashMap<Type, HashMap<String, Rc<Method>>>,
    local_variables: Vec<Value>,
    class_id_counter: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassID(usize);

impl fmt::Display for ClassID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl VM {
    pub fn new() -> Self {
        Self {
            methods: default_methods(),
            local_variables: Vec::new(),
            class_id_counter: 0,
        }
    }

    pub fn load_program(
        &mut self,
        program: Program,
    ) -> Result<HashMap<String, ClassID>> {
        let mut class_ids = HashMap::new();
        for class in program.classes {
            let class_id = self.new_class_id();
            class_ids.insert(class.name, class_id);
            for method in class.methods {
                let mut resolver = Resolver {
                    local_variables: std::iter::once("this".to_owned())
                        .chain(method.parameters)
                        .collect(),
                };
                let body = resolver.resolve_expression(method.body)?;
                self.methods
                    .entry(Type::Object(class_id))
                    .or_insert_with(Default::default)
                    .insert(
                        method.name.clone(),
                        Rc::new(Method::Custom { body }),
                    );
            }
        }
        Ok(class_ids)
    }

    pub fn run(&mut self, main_type: ClassID) -> Result<()> {
        let main_method = self
            .methods
            .get(&Type::Object(main_type))
            .and_then(|methods| methods.get("main"))
            .context("program has no entry point")?
            .clone();
        let this = Value::Object(Rc::new(Object {
            class: main_type,
            properties: HashMap::default(),
        }));
        self.invoke_method(&main_method, this, Vec::new())?;

        Ok(())
    }

    pub fn new_class_id(&mut self) -> ClassID {
        self.class_id_counter += 1;
        ClassID(self.class_id_counter)
    }

    fn invoke_method(
        &mut self,
        method: &Method,
        this: Value,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        match method {
            Method::Builtin(f) => Ok(f(self, &this, &arguments)),
            Method::Custom { body } => {
                let local_variable_count = self.local_variables.len();
                self.local_variables.push(this);
                self.local_variables.extend(arguments);
                let result = self.evaluate_expression(body);
                self.local_variables.truncate(local_variable_count);
                result
            }
        }
    }

    fn evaluate_expression(
        &mut self,
        expression: &Expression,
    ) -> Result<Value> {
        Ok(match expression {
            Expression::Literal(value) => value.clone(),
            Expression::MethodCall {
                name,
                this,
                arguments,
            } => {
                let this = self.evaluate_expression(this)?;
                let this_type = this.typ();
                let method = self
                    .methods
                    .get(&this_type)
                    .and_then(|methods| methods.get(name))
                    .with_context(|| {
                        format!(
                            "type `{this_type}` has no method named `{name}`"
                        )
                    })?
                    .clone();
                let arguments = arguments
                    .iter()
                    .map(|argument| self.evaluate_expression(argument))
                    .collect::<Result<_>>()?;
                self.invoke_method(&method, this, arguments)?
            }
            Expression::LocalVariable {
                name_or_de_bruijn_index: index,
            } => self
                .local_variables
                .get(self.local_variables.len() - 1 - *index)
                .with_context(|| {
                    format!("De Bruijn index {index} is out of range")
                })?
                .clone(),
            Expression::LetIn {
                name: (),
                bound,
                body,
            } => {
                let bound = self.evaluate_expression(bound)?;
                self.local_variables.push(bound);
                let result = self.evaluate_expression(body)?;
                self.local_variables.pop();
                result
            }
            Expression::IfThenElse {
                condition,
                if_true,
                if_false,
            } => {
                let Value::Bool(condition) = self.evaluate_expression(condition)? else { todo!() };
                self.evaluate_expression(if condition {
                    if_true
                } else {
                    if_false
                })?
            }
            Expression::Do(steps) => {
                let mut res = Value::Unit;
                for step in steps {
                    res = self.evaluate_expression(step)?;
                }
                res
            }
        })
    }
}
