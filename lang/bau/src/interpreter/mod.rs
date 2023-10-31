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
pub enum ControlFlowMode {
    Return(Option<Value>),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Interpreter {
    functions: HashMap<String, CheckedFunctionItem>,
    scope_stack: Vec<Scope>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            scope_stack: vec![],
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
                    name: function.definition.name.clone(),
                    expected_number: function.definition.parameters.len(),
                    found_number: arguments.len(),
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

        match self.evaluate_block(&function.body)? {
            Some(ControlFlowMode::Return(return_value)) => {
                self.pop_scope();
                Ok(return_value)
            }
            None => {
                self.pop_scope();
                Ok(None)
            }
        }
    }

    pub fn evaluate_statement(
        &mut self,
        statement: &CheckedStatement,
    ) -> ExecutionResult<Option<ControlFlowMode>> {
        match statement.kind() {
            CheckedStatementKind::Return { value } => {
                return match value {
                    Some(value_expression) => {
                        let return_value = self.evaluate_expression(value_expression)?;
                        Ok(Some(ControlFlowMode::Return(return_value)))
                    }
                    None => Ok(Some(ControlFlowMode::Return(None))),
                };
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
            }
            CheckedStatementKind::VariableAssignment {
                name,
                value,
                operator,
            } => self.evaluate_variable_assignment(name, value, operator)?,
            CheckedStatementKind::Expression { expression } => {
                self.evaluate_expression(expression)?;
            }
            CheckedStatementKind::If {
                condition,
                then_body,
                else_body,
            } => return self.evaluate_if_statement(condition, then_body, else_body.as_deref()),
            CheckedStatementKind::Loop { block } => loop {
                self.push_scope();
                if let Some(mode) = self.evaluate_block(block)? {
                    self.pop_scope();
                    return Ok(Some(mode));
                }
                self.pop_scope();
            },
        };
        Ok(None)
    }

    pub fn evaluate_block(
        &mut self,
        block: &[CheckedStatement],
    ) -> ExecutionResult<Option<ControlFlowMode>> {
        for statement in block {
            if let Some(control_flow_mode) = self.evaluate_statement(statement)? {
                return Ok(Some(control_flow_mode));
            }
        }
        Ok(None)
    }

    pub fn evaluate_variable_assignment(
        &mut self,
        name: &str,
        value: &CheckedExpression,
        operator: &TokenKind,
    ) -> ExecutionResult<()> {
        let value = match self.evaluate_expression(value)? {
            Some(value) => value,
            None => {
                return Err(ExecutionError::new(
                    ExecutionErrorKind::VariableDoesNotExist {
                        name: name.to_string(),
                    },
                ))
            }
        };

        let mut new_value = self.get_variable(name)?.clone();
        match operator {
            TokenKind::Equals => new_value = value,
            TokenKind::PlusEquals => new_value.add(value)?,
            TokenKind::MinusEquals => new_value.subtract(value)?,
            TokenKind::AsteriskEquals => new_value.multiply(value)?,
            TokenKind::SlashEquals => new_value.divide(value)?,
            TokenKind::PercentEquals => new_value.modulo(value)?,
            _ => panic!("Invalid compound assignment operator: {:?}", operator),
        };

        self.set_variable(name, new_value)
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
                let value = self.get_variable(&variable.name)?;
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
        let mut value = lhs.unwrap();
        let rhs = rhs.unwrap();

        match operator {
            TokenKind::Plus => value.add(rhs)?,
            TokenKind::Minus => value.subtract(rhs)?,
            TokenKind::Asterisk => value.multiply(rhs)?,
            TokenKind::Slash => value.divide(rhs)?,
            TokenKind::Percent => value.modulo(rhs)?,

            TokenKind::EqualsEquals => value.equals(rhs)?,
            TokenKind::ExclamationMarkEquals => value.not_equals(rhs)?,
            TokenKind::LessThan => value.less_than(rhs)?,
            TokenKind::GreaterThan => value.greater_than(rhs)?,
            TokenKind::LessThanEquals => value.less_than_equals(rhs)?,
            TokenKind::GreaterThanEquals => value.greater_than_equals(rhs)?,
            _ => panic!("Invalid infix operator: {:?}", operator),
        }

        return Ok(value);
    }

    fn evaluate_if_statement(
        &mut self,
        condition: &CheckedExpression,
        then_body: &[CheckedStatement],
        else_body: Option<&[CheckedStatement]>,
    ) -> ExecutionResult<Option<ControlFlowMode>> {
        let condition = self.evaluate_expression(condition)?.unwrap();
        if condition.is_true() {
            self.push_scope();
            if let Some(mode) = self.evaluate_block(then_body)? {
                self.pop_scope();
                return Ok(Some(mode));
            }
            self.pop_scope();
        } else if let Some(else_body) = else_body {
            self.push_scope();
            if let Some(mode) = self.evaluate_block(else_body)? {
                self.pop_scope();
                return Ok(Some(mode));
            }
            self.pop_scope();
        }

        Ok(None)
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

    fn get_variable(&self, name: &str) -> ExecutionResult<&Value> {
        for scope in self.scope_stack.iter().rev() {
            if let Ok(value) = scope.get_variable_by_name(name) {
                return Ok(value);
            }
        }
        Err(ExecutionError::new(
            ExecutionErrorKind::VariableDoesNotExist {
                name: name.to_string(),
            },
        ))
    }

    fn set_variable(&mut self, name: &str, value: Value) -> ExecutionResult<()> {
        for scope in self.scope_stack.iter_mut().rev() {
            if scope.set_variable(name, value.clone()).is_ok() {
                return Ok(());
            }
        }
        Err(ExecutionError::new(
            ExecutionErrorKind::VariableDoesNotExist {
                name: name.to_string(),
            },
        ))
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
