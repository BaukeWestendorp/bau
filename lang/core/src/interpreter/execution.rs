use crate::error::BauResult;
use crate::interpreter::scope::{ControlFlow, Scope};
use crate::interpreter::value::Value;
use crate::interpreter::Interpreter;
use crate::parser::ast::Literal;
use crate::tokenizer::token::TokenKind;
use crate::typechecker::{CheckedExpr, CheckedExprKind, CheckedFunctionItem, CheckedStmt};

#[macro_export]
macro_rules! execution_error {
    ($($message:tt)*) => {
        Err(crate::error::BauError::ExecutionError {
            message: format!($($message)*),
        })
    };
}

impl Interpreter {
    pub fn execute_main(&mut self) -> BauResult<Option<Value>> {
        match self.main_function().cloned() {
            Some(main) => self.execute_function(&main, &vec![]),
            None => execution_error!("No main function found"),
        }
    }

    pub fn execute_function(
        &mut self,
        function: &CheckedFunctionItem,
        _args: &Vec<CheckedExpr>,
    ) -> BauResult<Option<Value>> {
        let return_value =
            self.execute_block_statement(&function.body())?
                .map_or(None, |control_flow| match control_flow {
                    ControlFlow::Return(value) => value,
                    _ => None,
                });
        Ok(return_value)
    }

    pub fn execute_statement(&mut self, statement: &CheckedStmt) -> BauResult<()> {
        match statement {
            CheckedStmt::Let { .. } => self.execute_let_statement(statement),
            CheckedStmt::Assignment { .. } => self.execute_assignment_statement(statement),
            CheckedStmt::If { .. } => self.execute_if_statement(statement),
            CheckedStmt::Block { .. } => self.execute_block_statement(statement).map(|_| ()),
            CheckedStmt::Loop { .. } => self.execute_loop_statement(statement),
            CheckedStmt::Return { .. } => self.execute_return_statement(statement),
            CheckedStmt::Continue => self.execute_continue_statement(),
            CheckedStmt::Break => self.execute_break_statement(),
            CheckedStmt::Expression { .. } => self.execute_expression_statement(statement),
        }
    }

    pub fn execute_let_statement(&mut self, statement: &CheckedStmt) -> BauResult<()> {
        match statement {
            CheckedStmt::Let { name, expr, .. } => match self.execute_expression(expr)? {
                Some(initial_value) => {
                    self.set_variable_value(name, initial_value);
                    Ok(())
                }
                None => execution_error!("Variable can't be initialized to `void`"),
            },
            _ => panic!("Expected let statement, found: `{:?}`", statement),
        }
    }

    pub fn execute_assignment_statement(&mut self, statement: &CheckedStmt) -> BauResult<()> {
        match statement {
            CheckedStmt::Assignment { name, expr, .. } => {
                if let Some(value) = self.execute_expression(expr)? {
                    self.set_variable_value(name, value);
                    return Ok(());
                }
                execution_error!("Variable can't be assigned to `void`")
            }
            _ => panic!("Expected assignment statement, found: `{:?}`", statement),
        }
    }

    pub fn execute_if_statement(&mut self, statement: &CheckedStmt) -> BauResult<()> {
        match statement {
            CheckedStmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.execute_expression(condition)?;
                match condition {
                    Some(condition) => match condition {
                        Value::Bool(true) => self.execute_statement(then_branch),
                        Value::Bool(false) => match else_branch {
                            Some(else_branch) => self.execute_statement(else_branch),
                            None => Ok(()),
                        },
                        _ => execution_error!("Expected boolean condition, found: `{}`", condition),
                    },
                    _ => execution_error!("Expected boolean condition, found: `void`"),
                }
            }
            _ => panic!("Expected if statement, found: `{:?}`", statement),
        }
    }

    pub fn execute_block_statement(
        &mut self,
        statement: &CheckedStmt,
    ) -> BauResult<Option<ControlFlow>> {
        match statement {
            CheckedStmt::Block {
                statements,
                block_kind,
            } => {
                self.scope_stack.push(Scope {
                    control_flow: None,
                    block_kind: *block_kind,
                });
                for statement in statements {
                    self.execute_statement(statement)?;
                    if self.control_flow_should_break() {
                        break;
                    }
                }
                let control_flow = self.current_scope().control_flow.clone();
                self.scope_stack.pop();
                Ok(control_flow)
            }
            _ => panic!("Expected block statement, found: `{:?}`", statement),
        }
    }

    pub fn execute_loop_statement(&mut self, statement: &CheckedStmt) -> BauResult<()> {
        match statement {
            CheckedStmt::Loop { body } => loop {
                match self.execute_block_statement(body) {
                    Ok(control_flow) => match control_flow {
                        Some(ControlFlow::Continue) => continue,
                        Some(ControlFlow::Break) => break,
                        Some(ControlFlow::Return(_)) => return Ok(()),
                        None => {}
                    },
                    Err(error) => return Err(error),
                }
            },
            _ => panic!("Expected loop statement, found: `{:?}`", statement),
        }

        Ok(())
    }

    pub fn execute_return_statement(&mut self, statement: &CheckedStmt) -> BauResult<()> {
        match statement {
            CheckedStmt::Return { expr } => {
                let value = match expr {
                    Some(value) => value,
                    None => return Ok(()),
                };
                let value = self.execute_expression(value)?;
                self.set_control_flow(ControlFlow::Return(value.clone()));
                Ok(())
            }
            _ => panic!("Expected return statement, found: `{:?}`", statement),
        }
    }

    pub fn execute_continue_statement(&mut self) -> BauResult<()> {
        self.set_control_flow(ControlFlow::Continue);
        Ok(())
    }

    pub fn execute_break_statement(&mut self) -> BauResult<()> {
        self.set_control_flow(ControlFlow::Break);
        Ok(())
    }

    pub fn execute_expression_statement(&mut self, expression: &CheckedStmt) -> BauResult<()> {
        match expression {
            CheckedStmt::Expression { expr } => self.execute_expression(expr).map(|_| ()),
            _ => panic!("Expected expression statement, found: `{:?}`", expression),
        }
    }

    pub fn execute_expression(&mut self, expr: &CheckedExpr) -> BauResult<Option<Value>> {
        match &expr.kind() {
            CheckedExprKind::Literal(literal) => self.execute_literal_expression(literal),
            CheckedExprKind::Identifier(identifier) => {
                self.execute_identifier_expression(identifier)
            }
            CheckedExprKind::FnCall { .. } => self.execute_function_call_expression(expr),
            CheckedExprKind::PrefixOp { .. } => self.execute_prefix_operator_expression(expr),
            CheckedExprKind::InfixOp { .. } => self.execute_infix_operator_expression(expr),
            CheckedExprKind::PostfixOp { .. } => {
                execution_error!("PostfixOp expression execution not implemented")
            }
            CheckedExprKind::BuiltinFnCall { function, args } => function.call(self, args),
            CheckedExprKind::MethodCall(method) => self.execute_function(&method, &vec![]),
        }
    }

    pub fn execute_literal_expression(&mut self, literal: &Literal) -> BauResult<Option<Value>> {
        match literal {
            Literal::Int(value) => Ok(Some(Value::Int(*value))),
            Literal::Float(value) => Ok(Some(Value::Float(*value))),
            Literal::String(value) => Ok(Some(Value::String(value.to_string()))),
            Literal::Bool(value) => Ok(Some(Value::Bool(*value))),
        }
    }

    pub fn execute_identifier_expression(&mut self, ident: &str) -> BauResult<Option<Value>> {
        self.get_variable_value(ident).map(|v| v.cloned())
    }

    pub fn execute_function_call_expression(
        &mut self,
        function_call: &CheckedExpr,
    ) -> BauResult<Option<Value>> {
        match &function_call.kind() {
            CheckedExprKind::FnCall(call) => {
                let function = match self.functions.get(call.name()) {
                    Some(function) => function.clone(),
                    None => {
                        return execution_error!("No function found with name: `{}`", call.name())
                    }
                };

                let value = self.execute_function(&function, call.args())?;
                return Ok(value);
            }
            _ => panic!(
                "Expected function call expression, found: `{:?}`",
                function_call
            ),
        }
    }

    pub fn execute_prefix_operator_expression(
        &mut self,
        prefix_op: &CheckedExpr,
    ) -> BauResult<Option<Value>> {
        match &prefix_op.kind() {
            CheckedExprKind::PrefixOp { op, expr } => {
                let value = self.execute_expression(expr)?;
                match op {
                    TokenKind::Plus => Ok(value),
                    TokenKind::Minus => match value {
                        Some(Value::Int(value)) => Ok(Some(Value::Int(-value))),
                        Some(Value::Float(value)) => Ok(Some(Value::Float(-value))),
                        _ => execution_error!("Only ints and floats can be negated"),
                    },
                    TokenKind::ExclamationMark => match value {
                        Some(Value::Bool(value)) => Ok(Some(Value::Bool(!value))),
                        _ => execution_error!("Only bools can be negated"),
                    },
                    _ => execution_error!("Invalid prefix operator: `{}`", op),
                }
            }
            _ => panic!(
                "Expected prefix operator expression, found: `{:?}`",
                prefix_op
            ),
        }
    }

    pub fn execute_infix_operator_expression(
        &mut self,
        infix_op: &CheckedExpr,
    ) -> BauResult<Option<Value>> {
        match &infix_op.kind() {
            CheckedExprKind::InfixOp { op, lhs, rhs } => {
                let lhs = self.execute_expression(lhs)?;
                let rhs = self.execute_expression(rhs)?;
                if lhs.is_none() || rhs.is_none() {
                    return execution_error!("Infix operator can't be applied to `void`");
                }
                let lhs = lhs.unwrap();
                let rhs = rhs.unwrap();

                match op {
                    TokenKind::Plus => match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => Ok(Some(Value::Int(lhs + rhs))),
                        (Value::Float(lhs), Value::Float(rhs)) => Ok(Some(Value::Float(lhs + rhs))),
                        (Value::String(lhs), Value::String(rhs)) => {
                            Ok(Some(Value::String(format!("{}{}", lhs, rhs))))
                        }
                        _ => execution_error!(
                            "Addition is only available between ints, floats and strings"
                        ),
                    },
                    TokenKind::Minus => match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => Ok(Some(Value::Int(lhs - rhs))),
                        (Value::Float(lhs), Value::Float(rhs)) => Ok(Some(Value::Float(lhs - rhs))),
                        _ => execution_error!(
                            "Subtraction is only available between ints and floats"
                        ),
                    },
                    TokenKind::Asterisk => match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => Ok(Some(Value::Int(lhs * rhs))),
                        (Value::Float(lhs), Value::Float(rhs)) => Ok(Some(Value::Float(lhs * rhs))),
                        _ => execution_error!(
                            "Multiplication is only available between ints and floats"
                        ),
                    },
                    TokenKind::Slash => match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => Ok(Some(Value::Int(lhs / rhs))),
                        (Value::Float(lhs), Value::Float(rhs)) => Ok(Some(Value::Float(lhs / rhs))),
                        _ => execution_error!("Division is only available between ints and floats"),
                    },
                    TokenKind::Percent => match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => Ok(Some(Value::Int(lhs % rhs))),
                        _ => execution_error!("Modulo is only available between ints"),
                    },
                    TokenKind::EqualsEquals => Ok(Some(Value::Bool(lhs == rhs))),
                    TokenKind::ExclamationMarkEquals => Ok(Some(Value::Bool(lhs != rhs))),
                    TokenKind::LessThan => match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => Ok(Some(Value::Bool(lhs < rhs))),
                        (Value::Float(lhs), Value::Float(rhs)) => Ok(Some(Value::Bool(lhs < rhs))),
                        _ => {
                            execution_error!("Less than is only available between ints and floats")
                        }
                    },
                    TokenKind::LessThanEquals => match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => Ok(Some(Value::Bool(lhs <= rhs))),
                        (Value::Float(lhs), Value::Float(rhs)) => Ok(Some(Value::Bool(lhs <= rhs))),
                        _ => execution_error!(
                            "Less than or equals is only available between ints and floats"
                        ),
                    },
                    TokenKind::GreaterThan => match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => Ok(Some(Value::Bool(lhs > rhs))),
                        (Value::Float(lhs), Value::Float(rhs)) => Ok(Some(Value::Bool(lhs > rhs))),
                        _ => execution_error!(
                            "Greater than is only available between ints and floats"
                        ),
                    },
                    TokenKind::GreaterThanEquals => match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => Ok(Some(Value::Bool(lhs >= rhs))),
                        (Value::Float(lhs), Value::Float(rhs)) => Ok(Some(Value::Bool(lhs >= rhs))),
                        _ => execution_error!(
                            "Greater than or equals is only available between ints and floats"
                        ),
                    },
                    TokenKind::AmpersandAmpersand => match (lhs, rhs) {
                        (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Some(Value::Bool(lhs && rhs))),
                        _ => execution_error!("Logical and is only available between bools"),
                    },
                    TokenKind::PipePipe => match (lhs, rhs) {
                        (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Some(Value::Bool(lhs || rhs))),
                        _ => execution_error!("Logical or is only available between bools"),
                    },
                    _ => execution_error!("Invalid infix operator: `{}`", op),
                }
            }
            _ => panic!(
                "Expected infix operator expression, found: `{:?}`",
                infix_op
            ),
        }
    }
}
