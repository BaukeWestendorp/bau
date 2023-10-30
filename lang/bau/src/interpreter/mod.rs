use std::collections::HashMap;

use crate::typechecker::{
    CheckedExpression, CheckedFunctionItem, CheckedItem, CheckedItemKind, CheckedLiteralExpression,
    CheckedStatement, CheckedStatementKind,
};

pub mod error;
pub mod value;

use value::Value;

pub use error::ExecutionError;

use self::error::{ExecutionErrorKind, ExecutionResult};

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    variables: HashMap<String, Value>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn get_variable_by_name(&self, name: &str) -> ExecutionResult<&Value> {
        match self.variables.get(name) {
            Some(value) => Ok(value),
            None => Err(ExecutionError::new(
                ExecutionErrorKind::VariableDoesNotExist {
                    name: name.to_string(),
                },
            )),
        }
    }

    pub fn declare_variable(&mut self, name: &str, value: Value) -> ExecutionResult<()> {
        if self.variables.contains_key(name) {
            return Err(ExecutionError::new(
                ExecutionErrorKind::VariableAlreadyExists {
                    name: name.to_string(),
                },
            ));
        }

        self.variables.insert(name.to_string(), value);

        Ok(())
    }

    pub fn set_variable(&mut self, name: &str, value: Value) -> ExecutionResult<()> {
        if let Some(variable) = self.variables.get_mut(name) {
            *variable = value;
            Ok(())
        } else {
            Err(ExecutionError::new(
                ExecutionErrorKind::VariableDoesNotExist {
                    name: name.to_string(),
                },
            ))
        }
    }
}

type FunctionArguments = HashMap<String, Value>;

#[derive(Debug, Clone, PartialEq)]
enum ControlFlowMode {
    Return(Option<Value>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interpreter {
    functions: HashMap<String, CheckedFunctionItem>,
    scope_stack: Vec<Scope>,
    control_flow_mode: Option<ControlFlowMode>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            scope_stack: vec![],
            control_flow_mode: None,
        }
    }

    pub fn run(&mut self, checked_items: &[CheckedItem]) -> ExecutionResult<Option<Value>> {
        self.register_items(checked_items);

        let main_function = match self.main_function() {
            Some(main_function) => main_function.clone(),
            None => {
                return Err(ExecutionError::new(
                    ExecutionErrorKind::MainFunctionNotFound,
                ))
            }
        };

        self.execute_function(&main_function, HashMap::new())
    }

    fn execute_function(
        &mut self,
        function: &CheckedFunctionItem,
        arguments: FunctionArguments,
    ) -> ExecutionResult<Option<Value>> {
        self.push_scope();

        for argument in arguments.iter() {
            self.current_scope_mut()
                .declare_variable(argument.0, argument.1.clone())?;
        }

        for statement in &function.body {
            self.execute_statement(statement)?;

            if let Some(ControlFlowMode::Return(return_value)) = self.control_flow_mode.take() {
                self.control_flow_mode = None;
                self.pop_scope();
                return Ok(return_value);
            }
        }

        self.pop_scope();
        Ok(None)
    }

    fn execute_statement(&mut self, statement: &CheckedStatement) -> ExecutionResult<()> {
        match statement.kind() {
            CheckedStatementKind::Return { value } => {
                self.control_flow_mode = match value {
                    Some(value_expression) => {
                        let return_value = self.evaluate_expression(value_expression)?;
                        Some(ControlFlowMode::Return(return_value))
                    }
                    None => Some(ControlFlowMode::Return(None)),
                };
                Ok(())
            }
            CheckedStatementKind::Let {
                name,
                initial_value,
                ..
            } => {
                let value = match self.evaluate_expression(initial_value)? {
                    Some(initial_value) => initial_value,
                    None => {
                        return Err(ExecutionError::new(
                            ExecutionErrorKind::VariableDoesNotExist {
                                name: name.to_string(),
                            },
                        ))
                    }
                };

                self.current_scope_mut().declare_variable(name, value)?;

                Ok(())
            }
        }
    }

    fn evaluate_expression(
        &mut self,
        expression: &CheckedExpression,
    ) -> ExecutionResult<Option<Value>> {
        match expression {
            CheckedExpression::Literal(literal) => {
                let value = match literal {
                    CheckedLiteralExpression::Integer(value) => Value::Integer(*value),
                    CheckedLiteralExpression::String(value) => Value::String(value.clone()),
                    CheckedLiteralExpression::Boolean(value) => Value::Boolean(*value),
                    CheckedLiteralExpression::Float(value) => Value::Float(*value),
                };
                Ok(Some(value))
            }
            CheckedExpression::Variable(variable) => {
                let value = self
                    .current_scope_mut()
                    .get_variable_by_name(&variable.variable.name)?;
                Ok(Some(value.clone()))
            }
        }
    }

    fn register_items(&mut self, checked_items: &[CheckedItem]) {
        for item in checked_items {
            match item.kind() {
                CheckedItemKind::Function(function) => {
                    self.functions
                        .insert(function.name.clone(), function.clone());
                }
            }
        }
    }

    fn main_function(&self) -> Option<&CheckedFunctionItem> {
        self.functions.get("main")
    }

    fn push_scope(&mut self) {
        self.scope_stack.push(Scope::new());
    }

    fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        self.scope_stack.last_mut().unwrap()
    }
}
