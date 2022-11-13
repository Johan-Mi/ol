use crate::{value::Value, vm::ClassID};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Object {
    pub class: ClassID,
    pub properties: HashMap<String, Value>,
}
