use std::collections::HashMap;

use crate::tokenizer::token::TokenKind;
use crate::typechecker::{
    CheckedExpression, CheckedExpressionKind, CheckedFunctionDefinition, CheckedFunctionItem,
    CheckedItem, CheckedItemKind, CheckedLiteralExpression, CheckedStatement, CheckedStatementKind,
};

pub mod builtin;
pub mod error;
pub mod value;

use value::Value;

pub use error::ExecutionError;

use self::error::{ExecutionErrorKind, ExecutionResult};

#[derive(Debug, Clone, PartialEq, Default)]
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

#[derive(Debug, Clone, PartialEq)]
enum ControlFlowMode {
    Return(Option<Value>),
}

#[derive(Debug, Clone, PartialEq, Default)]
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
        for builtin_function in builtin::BUILTIN_FUNCTIONS.values() {
            self.register_function_definition(builtin_function, vec![]);
        }
        self.register_items(checked_items);

        let main_function = match self.main_function() {
            Some(main_function) => main_function.clone(),
            None => {
                return Err(ExecutionError::new(
                    ExecutionErrorKind::MainFunctionNotFound,
                ))
            }
        };

        self.evaluate_function(&main_function, vec![])
    }

    pub fn evaluate_function(
        &mut self,
        function: &CheckedFunctionItem,
        arguments: Vec<CheckedExpression>,
    ) -> ExecutionResult<Option<Value>> {
        self.push_scope();

        if arguments.len() != function.definition.parameters.len() {
            return Err(ExecutionError::new(
                ExecutionErrorKind::InvalidNumberOfArguments {
                    function: function.clone(),
                },
            ));
        };
        for (i, argument) in arguments.iter().enumerate() {
            if let Some(value) = self.evaluate_expression(argument)? {
                self.current_scope_mut()
                    .declare_variable(&function.definition.parameters[i].name, value)?;
            } else {
                return Err(ExecutionError::new(ExecutionErrorKind::InvalidArgument {
                    function: function.clone(),
                }));
            }
        }

        for statement in &function.body {
            self.evaluate_statement(statement)?;

            if let Some(ControlFlowMode::Return(return_value)) = self.control_flow_mode.take() {
                self.control_flow_mode = None;
                self.pop_scope();
                return Ok(return_value);
            }
        }

        self.pop_scope();
        Ok(None)
    }

    pub fn evaluate_statement(&mut self, statement: &CheckedStatement) -> ExecutionResult<()> {
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

    pub fn evaluate_expression(
        &mut self,
        expression: &CheckedExpression,
    ) -> ExecutionResult<Option<Value>> {
        match expression.kind() {
            CheckedExpressionKind::Literal(literal) => {
                let value = match literal {
                    CheckedLiteralExpression::Integer(value) => Value::Integer(*value),
                    CheckedLiteralExpression::String(value) => Value::String(value.clone()),
                    CheckedLiteralExpression::Boolean(value) => Value::Boolean(*value),
                    CheckedLiteralExpression::Float(value) => Value::Float(*value),
                };
                Ok(Some(value))
            }
            CheckedExpressionKind::Variable(variable) => {
                let value = self
                    .current_scope_mut()
                    .get_variable_by_name(&variable.name)?;
                Ok(Some(value.clone()))
            }
            CheckedExpressionKind::FunctionCall { name, arguments } => {
                if self.function_is_builtin(name.name()) {
                    return builtin::evaluate_builtin_function(self, name.name(), arguments);
                }

                let function = match self.get_function_by_name(name.name()) {
                    Some(function) => function.clone(),
                    None => {
                        return Err(ExecutionError::new(
                            ExecutionErrorKind::FunctionNotDefined {
                                name: name.name().to_string(),
                            },
                        ))
                    }
                };
                let return_value = self.evaluate_function(&function, arguments.clone())?;
                Ok(return_value)
            }
            CheckedExpressionKind::PrefixOperator {
                operator,
                expression,
            } => self
                .evaluate_prefix_operator(*operator, expression)
                .map(Some),
            CheckedExpressionKind::InfixOperator {
                operator,
                left,
                right,
            } => self
                .evaluate_infix_operator(*operator, left, right)
                .map(Some),
        }
    }

    pub fn evaluate_prefix_operator(
        &mut self,
        operator: TokenKind,
        expression: &CheckedExpression,
    ) -> ExecutionResult<Value> {
        let value = self.evaluate_expression(expression)?;
        if value.is_none() {
            return Err(ExecutionError::new(ExecutionErrorKind::InfixWithVoidSide));
        }
        let value = value.unwrap();

        match operator {
            TokenKind::Minus => match value {
                Value::Integer(value) => Ok(Value::Integer(-value)),
                Value::Float(value) => Ok(Value::Float(-value)),
                _ => Err(ExecutionError::new(
                    ExecutionErrorKind::PrefixWithInvalidType,
                )),
            },
            TokenKind::Plus => match value {
                Value::Integer(value) => Ok(Value::Integer(value)),
                Value::Float(value) => Ok(Value::Float(value)),
                _ => Err(ExecutionError::new(
                    ExecutionErrorKind::PrefixWithInvalidType,
                )),
            },
            TokenKind::ExclamationMark => match value {
                Value::Boolean(value) => Ok(Value::Boolean(!value)),
                _ => Err(ExecutionError::new(
                    ExecutionErrorKind::PrefixWithInvalidType,
                )),
            },
            _ => Err(ExecutionError::new(
                ExecutionErrorKind::PrefixWithInvalidType,
            )),
        }
    }

    pub fn evaluate_infix_operator(
        &mut self,
        operator: TokenKind,
        left: &CheckedExpression,
        right: &CheckedExpression,
    ) -> ExecutionResult<Value> {
        let lhs = self.evaluate_expression(left)?;
        let rhs = self.evaluate_expression(right)?;
        if lhs.is_none() || rhs.is_none() {
            return Err(ExecutionError::new(ExecutionErrorKind::InfixWithVoidSide));
        }
        let lhs = lhs.unwrap();
        let rhs = rhs.unwrap();

        match operator {
            TokenKind::Plus => match (lhs, rhs) {
                (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Integer(lhs + rhs)),
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs + rhs)),
                (Value::String(lhs), Value::String(rhs)) => Ok(Value::String(lhs + &rhs)),
                _ => Err(ExecutionError::new(
                    ExecutionErrorKind::InfixWithInvalidTypes,
                )),
            },
            TokenKind::Minus => match (lhs, rhs) {
                (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Integer(lhs - rhs)),
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs - rhs)),
                _ => Err(ExecutionError::new(
                    ExecutionErrorKind::InfixWithInvalidTypes,
                )),
            },
            TokenKind::Asterisk => match (lhs, rhs) {
                (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Integer(lhs * rhs)),
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs * rhs)),
                _ => Err(ExecutionError::new(
                    ExecutionErrorKind::InfixWithInvalidTypes,
                )),
            },
            TokenKind::Slash => match (lhs, rhs) {
                (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Integer(lhs / rhs)),
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs / rhs)),
                _ => Err(ExecutionError::new(
                    ExecutionErrorKind::InfixWithInvalidTypes,
                )),
            },
            TokenKind::EqualsEquals => match (lhs, rhs) {
                (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Boolean(lhs == rhs)),
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Boolean(lhs == rhs)),
                (Value::String(lhs), Value::String(rhs)) => Ok(Value::Boolean(lhs == rhs)),
                (Value::Boolean(lhs), Value::Boolean(rhs)) => Ok(Value::Boolean(lhs == rhs)),
                _ => Err(ExecutionError::new(
                    ExecutionErrorKind::InfixWithInvalidTypes,
                )),
            },
            TokenKind::ExclamationMarkEquals => match (lhs, rhs) {
                (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Boolean(lhs != rhs)),
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Boolean(lhs != rhs)),
                (Value::String(lhs), Value::String(rhs)) => Ok(Value::Boolean(lhs != rhs)),
                (Value::Boolean(lhs), Value::Boolean(rhs)) => Ok(Value::Boolean(lhs != rhs)),
                _ => Err(ExecutionError::new(
                    ExecutionErrorKind::InfixWithInvalidTypes,
                )),
            },
            TokenKind::LessThan => match (lhs, rhs) {
                (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Boolean(lhs < rhs)),
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Boolean(lhs < rhs)),
                (Value::String(lhs), Value::String(rhs)) => Ok(Value::Boolean(lhs < rhs)),
                _ => Err(ExecutionError::new(
                    ExecutionErrorKind::InfixWithInvalidTypes,
                )),
            },
            TokenKind::LessThanEquals => match (lhs, rhs) {
                (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Boolean(lhs <= rhs)),
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Boolean(lhs <= rhs)),
                (Value::String(lhs), Value::String(rhs)) => Ok(Value::Boolean(lhs <= rhs)),
                _ => Err(ExecutionError::new(
                    ExecutionErrorKind::InfixWithInvalidTypes,
                )),
            },
            TokenKind::GreaterThan => match (lhs, rhs) {
                (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Boolean(lhs > rhs)),
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Boolean(lhs > rhs)),
                (Value::String(lhs), Value::String(rhs)) => Ok(Value::Boolean(lhs > rhs)),
                _ => Err(ExecutionError::new(
                    ExecutionErrorKind::InfixWithInvalidTypes,
                )),
            },
            TokenKind::GreaterThanEquals => match (lhs, rhs) {
                (Value::Integer(lhs), Value::Integer(rhs)) => Ok(Value::Boolean(lhs >= rhs)),
                (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Boolean(lhs >= rhs)),
                (Value::String(lhs), Value::String(rhs)) => Ok(Value::Boolean(lhs >= rhs)),
                _ => Err(ExecutionError::new(
                    ExecutionErrorKind::InfixWithInvalidTypes,
                )),
            },
            _ => Err(ExecutionError::new(
                ExecutionErrorKind::InfixWithInvalidTypes,
            )),
        }
    }

    fn register_items(&mut self, checked_items: &[CheckedItem]) {
        for item in checked_items {
            match item.kind() {
                CheckedItemKind::Function(function) => {
                    self.register_function_definition(&function.definition, function.body.clone());
                }
            }
        }
    }

    fn register_function_definition(
        &mut self,
        function_definition: &CheckedFunctionDefinition,
        body: Vec<CheckedStatement>,
    ) {
        self.functions.insert(
            function_definition.name.clone(),
            CheckedFunctionItem {
                definition: function_definition.clone(),
                body,
            },
        );
    }

    pub fn main_function(&self) -> Option<&CheckedFunctionItem> {
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

    fn get_function_by_name(&self, name: &str) -> Option<&CheckedFunctionItem> {
        self.functions.get(name)
    }

    fn function_is_builtin(&self, name: &str) -> bool {
        builtin::BUILTIN_FUNCTIONS.contains_key(name)
    }
}
