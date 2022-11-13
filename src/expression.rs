use crate::value::Value;

pub type Expression = ExpressionOf<(), usize>;

#[derive(Debug, Clone)]
pub enum ExpressionOf<NewVar, GetVar> {
    Literal(Value),
    MethodCall {
        name: String,
        this: Box<Self>,
        arguments: Vec<Self>,
    },
    LocalVariable {
        name_or_de_brujin_index: GetVar,
    },
    LetIn {
        name: NewVar,
        bound: Box<Self>,
        body: Box<Self>,
    },
    IfThenElse {
        condition: Box<Self>,
        if_true: Box<Self>,
        if_false: Box<Self>,
    },
    Do(Vec<Self>),
}

impl ExpressionOf<String, String> {
    pub fn resolve(self) -> Expression {
        Resolver {
            local_variables: Vec::new(),
        }
        .resolve_expression(self)
    }
}

struct Resolver {
    local_variables: Vec<String>,
}

impl Resolver {
    fn resolve_expression(
        &mut self,
        expression: ExpressionOf<String, String>,
    ) -> Expression {
        match expression {
            ExpressionOf::Literal(value) => Expression::Literal(value),
            ExpressionOf::MethodCall {
                name,
                this,
                arguments,
            } => ExpressionOf::MethodCall {
                name,
                this: Box::new(self.resolve_expression(*this)),
                arguments: arguments
                    .into_iter()
                    .map(|argument| self.resolve_expression(argument))
                    .collect(),
            },
            ExpressionOf::LocalVariable {
                name_or_de_brujin_index: name,
            } => Expression::LocalVariable {
                name_or_de_brujin_index: self
                    .local_variables
                    .iter()
                    .rev()
                    .position(|variable| *variable == name)
                    .unwrap(),
            },
            ExpressionOf::LetIn { name, bound, body } => {
                self.local_variables.push(name);
                let result = Expression::LetIn {
                    name: (),
                    bound: Box::new(self.resolve_expression(*bound)),
                    body: Box::new(self.resolve_expression(*body)),
                };
                self.local_variables.pop();
                result
            }
            ExpressionOf::IfThenElse {
                condition,
                if_true,
                if_false,
            } => Expression::IfThenElse {
                condition: Box::new(condition.resolve()),
                if_true: Box::new(if_true.resolve()),
                if_false: Box::new(if_false.resolve()),
            },
            ExpressionOf::Do(steps) => Expression::Do(
                steps
                    .into_iter()
                    .map(|step| self.resolve_expression(step))
                    .collect(),
            ),
        }
    }
}
