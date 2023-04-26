use crate::expression::{Expression, ExpressionOf};
use anyhow::{anyhow, Result};

pub struct Resolver {
    pub local_variables: Vec<String>,
}

impl Resolver {
    pub fn resolve_expression(
        &mut self,
        expression: ExpressionOf<String, String>,
    ) -> Result<Expression> {
        Ok(match expression {
            ExpressionOf::Literal(value) => Expression::Literal(value),
            ExpressionOf::MethodCall {
                name,
                this,
                arguments,
            } => ExpressionOf::MethodCall {
                name,
                this: Box::new(self.resolve_expression(*this)?),
                arguments: arguments
                    .into_iter()
                    .map(|argument| self.resolve_expression(argument))
                    .collect::<Result<_>>()?,
            },
            ExpressionOf::LocalVariable {
                name_or_de_brujin_index: name,
            } => Expression::LocalVariable {
                name_or_de_brujin_index: self
                    .local_variables
                    .iter()
                    .rev()
                    .position(|variable| *variable == name)
                    .ok_or_else(|| {
                        anyhow!("variable `{name}` is not defined")
                    })?,
            },
            ExpressionOf::LetIn { name, bound, body } => {
                self.local_variables.push(name);
                let result = Expression::LetIn {
                    name: (),
                    bound: Box::new(self.resolve_expression(*bound)?),
                    body: Box::new(self.resolve_expression(*body)?),
                };
                self.local_variables.pop();
                result
            }
            ExpressionOf::IfThenElse {
                condition,
                if_true,
                if_false,
            } => Expression::IfThenElse {
                condition: Box::new(self.resolve_expression(*condition)?),
                if_true: Box::new(self.resolve_expression(*if_true)?),
                if_false: Box::new(self.resolve_expression(*if_false)?),
            },
            ExpressionOf::Do(steps) => Expression::Do(
                steps
                    .into_iter()
                    .map(|step| self.resolve_expression(step))
                    .collect::<Result<_>>()?,
            ),
        })
    }
}
