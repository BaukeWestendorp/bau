use crate::parser::ast::Item;
use std::collections::HashMap;

pub mod evaluation;
pub mod execution;
pub mod value;

pub struct Interpreter {
    functions: HashMap<String, Item>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }
}
