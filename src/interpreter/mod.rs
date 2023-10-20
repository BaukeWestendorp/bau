use crate::builtins::BUILTIN_FUNCTIONS;
use crate::error::BauResult;
use crate::execution_error;
use crate::interpreter::scope::{ControlFlow, Scope};
use crate::interpreter::value::Value;
use crate::parser::ast::Item;
use std::collections::HashMap;

pub mod evaluation;
pub mod execution;
pub mod scope;
pub mod value;

pub struct Interpreter {
    functions: HashMap<String, Item>,
    variables: HashMap<String, Value>,
    scope_stack: Vec<Scope>,
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
            scope_stack: vec![],
        }
    }

    pub fn main_function(&mut self) -> BauResult<&Item> {
        match self.functions.get(MAIN_FUNCTION_NAME) {
            Some(main) => Ok(main),
            None => execution_error!("No main function found"),
        }
    }

    pub fn current_scope(&mut self) -> &Scope {
        self.scope_stack
            .last()
            .expect("Scope stack should not be empty")
    }

    pub fn set_control_flow(&mut self, control_flow: ControlFlow) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.control_flow = Some(control_flow);
        }
    }

    pub fn control_flow_should_break(&mut self) -> bool {
        self.current_scope().control_flow.is_some()
    }
}
