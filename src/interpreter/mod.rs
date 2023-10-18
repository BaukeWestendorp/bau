use crate::error::{BauError, BauResult};
use crate::parser::ast::Item;
use std::collections::HashMap;

pub mod evaluation;
pub mod execution;
pub mod value;

pub struct Interpreter {
    functions: HashMap<String, Item>,
}

const MAIN_FUNCTION_NAME: &str = "main";
const PRINT_FUNCTION_NAME: &str = "print";

impl Interpreter {
    pub fn new() -> Self {
        let mut functions = HashMap::new();

        let print_function = Item::Function {
            name: PRINT_FUNCTION_NAME.to_string(),
            parameters: vec!["value".to_string()],
            body: vec![],
        };

        functions.insert(PRINT_FUNCTION_NAME.to_string(), print_function);

        Self { functions }
    }

    pub fn main_function(&mut self) -> BauResult<&Item> {
        match self.functions.get(MAIN_FUNCTION_NAME) {
            Some(main) => Ok(main),
            None => Err(BauError::ExecutionError {
                message: "No main function found".to_string(),
            }),
        }
    }
}
