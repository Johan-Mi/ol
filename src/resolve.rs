use crate::expression::{self, Expression};
use anyhow::{anyhow, Result};

pub struct Resolver {
    pub local_variables: Vec<String>,
}

impl Resolver {
    pub fn resolve_expression(
        &mut self,
        expression: expression::Of<String, String>,
    ) -> Result<Expression> {
        Ok(match expression {
            expression::Of::Literal(value) => expression::Of::Literal(value),
            expression::Of::MethodCall {
                name,
                this,
                arguments,
            } => expression::Of::MethodCall {
                name,
                this: Box::new(self.resolve_expression(*this)?),
                arguments: arguments
                    .into_iter()
                    .map(|argument| self.resolve_expression(argument))
                    .collect::<Result<_>>()?,
            },
            expression::Of::LocalVariable {
                name_or_de_bruijn_index: name,
            } => expression::Of::LocalVariable {
                name_or_de_bruijn_index: self
                    .local_variables
                    .iter()
                    .rev()
                    .position(|variable| *variable == name)
                    .ok_or_else(|| {
                        anyhow!("variable `{name}` is not defined")
                    })?,
            },
            expression::Of::LetIn { name, bound, body } => {
                self.local_variables.push(name);
                let result = expression::Of::LetIn {
                    name: (),
                    bound: Box::new(self.resolve_expression(*bound)?),
                    body: Box::new(self.resolve_expression(*body)?),
                };
                self.local_variables.pop();
                result
            }
            expression::Of::IfThenElse {
                condition,
                if_true,
                if_false,
            } => expression::Of::IfThenElse {
                condition: Box::new(self.resolve_expression(*condition)?),
                if_true: Box::new(self.resolve_expression(*if_true)?),
                if_false: Box::new(self.resolve_expression(*if_false)?),
            },
            expression::Of::Do(steps) => expression::Of::Do(
                steps
                    .into_iter()
                    .map(|step| self.resolve_expression(step))
                    .collect::<Result<_>>()?,
            ),
        })
    }
}
