use crate::builtins::BUILTIN_FUNCTIONS;
use crate::error::BauResult;
use crate::execution_error;
use crate::interpreter::value::Value;
use crate::parser::ast::Item;
use std::collections::HashMap;

pub mod evaluation;
pub mod execution;
pub mod value;

pub struct Interpreter {
    functions: HashMap<String, Item>,
    variables: HashMap<String, Value>,
}

const MAIN_FUNCTION_NAME: &str = "main";

impl Interpreter {
    pub fn new() -> Self {
        let mut functions = HashMap::new();
        for builtin in BUILTIN_FUNCTIONS.iter() {
            functions.insert(builtin.name(), builtin.function.clone());
        }
        Self {
            functions,
            variables: HashMap::new(),
        }
    }

    pub fn main_function(&mut self) -> BauResult<&Item> {
        match self.functions.get(MAIN_FUNCTION_NAME) {
            Some(main) => Ok(main),
            None => execution_error!("No main function found"),
        }
    }
}
