use crate::builtins::BUILTIN_FUNCTIONS;
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

impl Interpreter {
    pub fn new() -> Self {
        let mut functions = HashMap::new();
        for builtin in BUILTIN_FUNCTIONS.iter() {
            functions.insert(builtin.name(), builtin.function.clone());
        }
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
