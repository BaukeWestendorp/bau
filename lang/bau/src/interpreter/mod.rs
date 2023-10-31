use std::collections::HashMap;

use crate::parser::{AssignmentOperator, Identifier, PrefixOperator};
use crate::tokenizer::token::TokenKind;
use crate::typechecker::{
    CheckedExpression, CheckedExpressionKind, CheckedFunctionDefinition, CheckedFunctionItem,
    CheckedItem, CheckedItemKind, CheckedStatement, CheckedStatementKind, CheckedVariable,
};

pub mod builtin;
pub mod error;
pub mod value;

use value::Value;

use self::error::ExecutionResult;

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

    pub fn get_variable(&self, name: &str) -> &Value {
        match self.variables.get(name) {
            Some(value) => value,
            None => panic!("Variable with name `{}` not found", name),
        }
    }

    pub fn set_variable(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
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

        let main_function = self.main_function().clone();
        self.evaluate_function(&main_function, &[])
    }

    pub fn evaluate_function(
        &mut self,
        function: &CheckedFunctionItem,
        arguments: &[CheckedExpression],
    ) -> ExecutionResult<Option<Value>> {
        self.push_scope();

        assert_eq!(
            function.definition.parameters.len(),
            arguments.len(),
            "Typechecker should have checked argument counts. Expected {} arguments, but found {}",
            function.definition.parameters.len(),
            arguments.len(),
        );

        for (i, argument) in arguments.iter().enumerate() {
            let value = self
                .evaluate_expression(argument)?
                .expect("Typechecker should have checked for void expressions in function call");
            self.current_scope_mut()
                .set_variable(&function.definition.parameters[i].name, value);
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
            CheckedStatementKind::Return { value } => return self.evaluate_return_statement(value),
            CheckedStatementKind::Let {
                name,
                initial_value,
                ..
            } => self.evaluate_let_statement(name, initial_value)?,
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
            CheckedStatementKind::Loop { block } => return self.evaluate_loop_statement(block),
            CheckedStatementKind::While { condition, block } => {
                return self.evaluate_while_statement(condition, block)
            }
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

    pub fn evaluate_return_statement(
        &mut self,
        value: &Option<CheckedExpression>,
    ) -> ExecutionResult<Option<ControlFlowMode>> {
        match value {
            Some(value_expression) => {
                let return_value = self.evaluate_expression(value_expression)?;
                Ok(Some(ControlFlowMode::Return(return_value)))
            }
            None => Ok(Some(ControlFlowMode::Return(None))),
        }
    }

    pub fn evaluate_let_statement(
        &mut self,
        name: &str,
        initial_value: &CheckedExpression,
    ) -> ExecutionResult<()> {
        let value = self
            .evaluate_expression(initial_value)?
            .expect("Typechecker should have checked for void expressions in variable assignment");
        self.current_scope_mut().set_variable(name, value);
        Ok(())
    }

    pub fn evaluate_variable_assignment(
        &mut self,
        name: &str,
        value: &CheckedExpression,
        operator: &AssignmentOperator,
    ) -> ExecutionResult<()> {
        let value = self
            .evaluate_expression(value)?
            .expect("Typechecker should have checked for void expressions in variable assignment");

        let mut new_value = self.get_variable(name).clone();
        match operator {
            AssignmentOperator::Equals => new_value = value,
            AssignmentOperator::PlusEquals => new_value.add(value),
            AssignmentOperator::MinusEquals => new_value.subtract(value),
            AssignmentOperator::AsteriskEquals => new_value.multiply(value),
            AssignmentOperator::SlashEquals => new_value.divide(value),
            AssignmentOperator::PercentEquals => new_value.modulo(value),
        };

        self.set_variable(name, new_value);
        Ok(())
    }

    pub fn evaluate_expression(
        &mut self,
        expression: &CheckedExpression,
    ) -> ExecutionResult<Option<Value>> {
        match expression.kind() {
            CheckedExpressionKind::Literal(literal) => Ok(Some(literal.clone())),
            CheckedExpressionKind::Variable(variable) => self.evaluate_variable(variable),
            CheckedExpressionKind::FunctionCall { name, arguments } => {
                self.evaluate_function_call(name, arguments)
            }
            CheckedExpressionKind::PrefixOperator {
                operator,
                expression,
            } => self
                .evaluate_prefix_operator(operator, expression)
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

    pub fn evaluate_variable(&self, variable: &CheckedVariable) -> ExecutionResult<Option<Value>> {
        let value = self.get_variable(&variable.name);
        Ok(Some(value.clone()))
    }

    pub fn evaluate_function_call(
        &mut self,
        name: &Identifier,
        arguments: &[CheckedExpression],
    ) -> ExecutionResult<Option<Value>> {
        if self.function_is_builtin(name.name()) {
            return builtin::evaluate_builtin_function(self, name.name(), arguments);
        }

        let function = self.get_function(name.name()).clone();
        self.evaluate_function(&function, arguments.clone())
    }

    pub fn evaluate_prefix_operator(
        &mut self,
        operator: &PrefixOperator,
        expression: &CheckedExpression,
    ) -> ExecutionResult<Value> {
        let value = self
            .evaluate_expression(expression)?
            .expect("Typechecker should have checked for void expressions");

        match operator {
            PrefixOperator::Minus => match value {
                Value::Integer(value) => Ok(Value::Integer(-value)),
                Value::Float(value) => Ok(Value::Float(-value)),
                _ => panic!("Typechecker should have checked for invalid prefix operands"),
            },
            PrefixOperator::Plus => match value {
                Value::Integer(value) => Ok(Value::Integer(value)),
                Value::Float(value) => Ok(Value::Float(value)),
                _ => panic!("Typechecker should have checked for invalid prefix operands"),
            },
            PrefixOperator::ExclamationMark => match value {
                Value::Boolean(value) => Ok(Value::Boolean(!value)),
                _ => panic!("Typechecker should have checked for invalid prefix operands"),
            },
        }
    }

    pub fn evaluate_infix_operator(
        &mut self,
        operator: TokenKind,
        left: &CheckedExpression,
        right: &CheckedExpression,
    ) -> ExecutionResult<Value> {
        let lhs = self
            .evaluate_expression(left)?
            .expect("Typechecker should have checked for void expressions");
        let rhs = self
            .evaluate_expression(right)?
            .expect("Typechecker should have checked for void expressions");
        let mut value = lhs;

        match operator {
            TokenKind::Plus => value.add(rhs),
            TokenKind::Minus => value.subtract(rhs),
            TokenKind::Asterisk => value.multiply(rhs),
            TokenKind::Slash => value.divide(rhs),
            TokenKind::Percent => value.modulo(rhs),

            TokenKind::EqualsEquals => value.equals(rhs),
            TokenKind::ExclamationMarkEquals => value.not_equals(rhs),
            TokenKind::LessThan => value.less_than(rhs),
            TokenKind::GreaterThan => value.greater_than(rhs),
            TokenKind::LessThanEquals => value.less_than_equals(rhs),
            TokenKind::GreaterThanEquals => value.greater_than_equals(rhs),
            _ => panic!("Invalid infix operator: {:?}", operator),
        }

        Ok(value)
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

    fn evaluate_loop_statement(
        &mut self,
        block: &[CheckedStatement],
    ) -> ExecutionResult<Option<ControlFlowMode>> {
        loop {
            self.push_scope();
            if let Some(mode) = self.evaluate_block(block)? {
                self.pop_scope();
                return Ok(Some(mode));
            }
            self.pop_scope();
        }
    }

    fn evaluate_while_statement(
        &mut self,
        condition: &CheckedExpression,
        block: &[CheckedStatement],
    ) -> ExecutionResult<Option<ControlFlowMode>> {
        loop {
            let condition = self.evaluate_expression(condition)?.unwrap();
            if condition.is_false() {
                break;
            }

            self.push_scope();
            if let Some(mode) = self.evaluate_block(block)? {
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

    pub fn main_function(&self) -> &CheckedFunctionItem {
        self.functions
            .get("main")
            .expect("Typechecker should have checked for main function")
    }

    fn push_scope(&mut self) {
        self.scope_stack.push(Scope::new());
    }

    fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn get_variable(&self, name: &str) -> &Value {
        for scope in self.scope_stack.iter().rev() {
            if scope.has_variable(name) {
                return scope.get_variable(name);
            }
        }
        panic!("Variable with name `{}` not found", name);
    }

    fn set_variable(&mut self, name: &str, value: Value) {
        for scope in self.scope_stack.iter_mut().rev() {
            if scope.has_variable(name) {
                scope.set_variable(name, value);
                return;
            }
        }
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        self.scope_stack.last_mut().unwrap()
    }

    fn get_function(&self, name: &str) -> &CheckedFunctionItem {
        self.functions
            .get(name)
            .expect("Typechecker should have checked if function exists")
    }

    fn function_is_builtin(&self, name: &str) -> bool {
        builtin::BUILTIN_FUNCTIONS.contains_key(name)
    }
}
