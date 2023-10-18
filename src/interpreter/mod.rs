use crate::error::{BauError, BauResult};
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
        let mut functions = HashMap::new();

        let print_function = Item::Function {
            name: "print".to_string(),
            parameters: vec!["value".to_string()],
            body: vec![],
        };

        functions.insert("print".to_string(), print_function);

        Self { functions }
    }

    pub fn main_function(&mut self) -> BauResult<&Item> {
        match self.functions.get("main") {
            Some(main) => Ok(main),
            None => Err(BauError::ExecutionError {
                message: "No main function found".to_string(),
            }),
        }
    }
}
