use crate::builtins::BUILTIN_FUNCTIONS;
use crate::interpreter::scope::{ControlFlow, Scope};
use crate::interpreter::value::Value;
use crate::parser::ast::{BlockKind, Item, Type};
use std::collections::HashMap;

pub mod evaluation;
pub mod execution;
pub mod scope;
pub mod value;

pub struct Variable {
    name: String,
    var_type: Type,
    value: Value,
}

pub struct Interpreter {
    functions: HashMap<String, Item>,
    variables: HashMap<String, Variable>,
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

    pub fn main_function(&self) -> Option<&Item> {
        self.functions.get(MAIN_FUNCTION_NAME)
    }

    pub fn set_variable_value(&mut self, name: &str, value: Value) {
        if let Some(variable) = self.variables.get_mut(name) {
            variable.value = value;
        }
    }

    pub fn current_scope(&self) -> &Scope {
        self.scope_stack
            .last()
            .expect("Scope stack should not be empty")
    }

    pub fn current_scope_mut(&mut self) -> &mut Scope {
        self.scope_stack
            .last_mut()
            .expect("Scope stack should not be empty")
    }

    pub fn current_loop_scope(&self) -> Option<&Scope> {
        for scope in self.scope_stack.iter().rev() {
            if scope.block_kind == BlockKind::Loop {
                return Some(scope);
            }
        }
        None
    }

    pub fn current_loop_scope_mut(&mut self) -> Option<&mut Scope> {
        for scope in self.scope_stack.iter_mut().rev() {
            if scope.block_kind == BlockKind::Loop {
                return Some(scope);
            }
        }
        None
    }

    pub fn set_control_flow(&mut self, control_flow: ControlFlow) {
        match control_flow {
            ControlFlow::Break | ControlFlow::Continue => {
                if let Some(scope) = self.current_loop_scope_mut() {
                    scope.control_flow = Some(control_flow);
                }
            }
            ControlFlow::Return(_) => {
                self.current_scope_mut().control_flow = Some(control_flow);
            }
        }
    }

    pub fn control_flow_should_break(&mut self) -> bool {
        self.current_scope().control_flow.is_some()
    }
}
