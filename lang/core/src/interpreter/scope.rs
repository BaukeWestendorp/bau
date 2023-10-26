use std::collections::HashMap;

use crate::error::BauResult;
use crate::execution_error;
use crate::interpreter::value::Value;
use crate::parser::ast::BlockKind;

#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    Continue,
    Break,
    Return(Option<Value>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub control_flow: Option<ControlFlow>,
    pub block_kind: BlockKind,

    variables: HashMap<String, Value>,
}

impl Scope {
    pub fn new(block_kind: BlockKind) -> Self {
        Self {
            control_flow: None,
            block_kind: block_kind,
            variables: HashMap::new(),
        }
    }
    pub fn variable_exists(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    pub fn get_variable_value(&self, name: &str) -> BauResult<Option<&Value>> {
        match self.variables.get(name) {
            Some(var) => Ok(Some(var)),
            None => execution_error!("No variable found with name: `{}`", name),
        }
    }

    pub fn set_variable_value(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }
}
