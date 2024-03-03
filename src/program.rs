#[derive(Debug)]
pub struct Program {
    pub classes: Vec<Class>,
}

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub methods: Vec<ClassMethod>,
}

#[derive(Debug)]
pub struct ClassMethod {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: crate::expression::Of<String, String>,
}
