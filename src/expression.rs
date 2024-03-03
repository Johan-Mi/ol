use crate::value::Value;

pub type Expression = Of<(), usize>;

#[derive(Debug, Clone)]
pub enum Of<NewVar, GetVar> {
    Literal(Value),
    MethodCall {
        name: String,
        this: Box<Self>,
        arguments: Vec<Self>,
    },
    LocalVariable {
        name_or_de_bruijn_index: GetVar,
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
