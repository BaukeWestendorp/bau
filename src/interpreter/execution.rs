use crate::error::BauResult;
use crate::interpreter::scope::{ControlFlow, Scope};
use crate::interpreter::value::Value;
use crate::interpreter::Interpreter;
use crate::parser::ast::{Expr, ExprKind, Item, Literal, Stmt};
use crate::tokenizer::token::TokenKind;

macro_rules! execution_error {
    ($($message:tt)*) => {
        Err(crate::error::BauError::ExecutionError {
            message: format!($($message)*),
        })
    };
}

impl Interpreter {
    pub fn execute_main(&mut self) -> BauResult<()> {
        match self.main_function().cloned() {
            Some(main) => {
                self.execute_function(&main, &vec![])?;
                Ok(())
            }
            None => execution_error!("No main function found"),
        }
    }

    pub fn execute_function(
        &mut self,
        function: &Item,
        _args: &Vec<Expr>,
    ) -> BauResult<Option<Value>> {
        match function {
            Item::Function { body, .. } => {
                let return_value =
                    self.execute_block_statement(body)?
                        .map_or(None, |control_flow| match control_flow {
                            ControlFlow::Return(value) => value,
                            _ => None,
                        });
                Ok(return_value)
            }
        }
    }

    pub fn execute_statement(&mut self, statement: &Stmt) -> BauResult<()> {
        match statement {
            Stmt::Let { .. } => self.execute_let_statement(statement),
            Stmt::Assignment { .. } => self.execute_assignment_statement(statement),
            Stmt::If { .. } => self.execute_if_statement(statement),
            Stmt::Block { .. } => {
                self.execute_block_statement(statement)?;
                Ok(())
            }
            Stmt::Loop { .. } => self.execute_loop_statement(statement),
            Stmt::Return { .. } => self.execute_return_statement(statement),
            Stmt::Continue => self.execute_continue_statement(),
            Stmt::Break => self.execute_break_statement(),
            Stmt::Expression { .. } => self.execute_expression_statement(statement),
        }
    }

    pub fn execute_let_statement(&mut self, statement: &Stmt) -> BauResult<()> {
        match statement {
            Stmt::Let { name, expr, .. } => {
                let initial_value = self.execute_expression(expr)?;
                self.variables.insert(name.clone(), initial_value);
                Ok(())
            }
            _ => panic!("Expected let statement"),
        }
    }

    pub fn execute_assignment_statement(&mut self, statement: &Stmt) -> BauResult<()> {
        match statement {
            Stmt::Assignment { name, expr, .. } => {
                let value = self.execute_expression(expr)?;
                if !self.variables.contains_key(name) {
                    return execution_error!("No variable found with name: `{}`", name);
                }
                self.set_variable_value(name, value);
                Ok(())
            }
            _ => panic!("Expected assignment statement"),
        }
    }

    pub fn execute_if_statement(&mut self, statement: &Stmt) -> BauResult<()> {
        match statement {
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.execute_expression(condition)?;
                match condition {
                    Value::Bool(true) => self.execute_statement(then_branch),
                    Value::Bool(false) => match else_branch {
                        Some(else_branch) => self.execute_statement(else_branch),
                        None => Ok(()),
                    },
                    _ => execution_error!("Expected boolean condition, found: `{}`", condition),
                }
            }
            _ => panic!("Expected if statement"),
        }
    }

    pub fn execute_block_statement(&mut self, statement: &Stmt) -> BauResult<Option<ControlFlow>> {
        match statement {
            Stmt::Block {
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
            _ => panic!("Expected block statement"),
        }
    }

    pub fn execute_loop_statement(&mut self, statement: &Stmt) -> BauResult<()> {
        match statement {
            Stmt::Loop { body } => loop {
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
            _ => panic!("Expected loop statement"),
        }

        Ok(())
    }

    pub fn execute_return_statement(&mut self, statement: &Stmt) -> BauResult<()> {
        match statement {
            Stmt::Return { expr } => {
                let value = match expr {
                    Some(value) => value,
                    None => return Ok(()),
                };
                let value = self.execute_expression(value)?;
                self.set_control_flow(ControlFlow::Return(Some(value.clone())));
                Ok(())
            }
            _ => panic!("Expected return statement"),
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

    pub fn execute_expression_statement(&mut self, expression: &Stmt) -> BauResult<()> {
        match expression {
            Stmt::Expression { expr } => self.execute_expression(expr).map(|_| ()),
            _ => panic!("Expected expression statement"),
        }
    }

    pub fn execute_expression(&mut self, expr: &Expr) -> BauResult<Value> {
        match &expr.kind {
            ExprKind::Literal(literal) => self.execute_literal_expression(literal),
            ExprKind::Identifier(identifier) => self.execute_identifier_expression(identifier),
            ExprKind::FnCall { .. } => self.execute_function_call_expression(expr),
            ExprKind::PrefixOp { .. } => self.execute_prefix_operator_expression(expr),
            ExprKind::InfixOp { .. } => self.execute_infix_operator_expression(expr),
            ExprKind::PostfixOp { .. } => {
                execution_error!("PostfixOp expression execution not implemented")
            }
            ExprKind::BuiltinFnCall { function, args } => function.call(self, args),
        }
    }

    pub fn execute_literal_expression(&mut self, literal: &Literal) -> BauResult<Value> {
        match literal {
            Literal::Int(value) => Ok(Value::Int(*value)),
            Literal::Float(value) => Ok(Value::Float(*value)),
            Literal::String(value) => Ok(Value::String(value.clone())),
            Literal::Bool(value) => Ok(Value::Bool(*value)),
        }
    }

    pub fn execute_identifier_expression(&mut self, ident: &str) -> BauResult<Value> {
        match self.variables.get(ident) {
            Some(var) => Ok(var.clone()),
            None => execution_error!("No variable found with name: `{}`", ident),
        }
    }

    pub fn execute_function_call_expression(&mut self, function_call: &Expr) -> BauResult<Value> {
        match &function_call.kind {
            ExprKind::FnCall { name, args } => {
                let function = match self.functions.get(name) {
                    Some(function) => function.clone(),
                    None => return execution_error!("No function found with name: `{}`", name),
                };

                let value = self.execute_function(&function, args)?;
                return Ok(value.unwrap_or(Value::none()));
            }
            _ => panic!("Expected function call expression"),
        }
    }

    pub fn execute_prefix_operator_expression(&mut self, prefix_op: &Expr) -> BauResult<Value> {
        match &prefix_op.kind {
            ExprKind::PrefixOp { op, expr } => {
                let value = self.execute_expression(expr)?;
                match op {
                    TokenKind::Plus => Ok(value),
                    TokenKind::Minus => match value {
                        Value::Int(value) => Ok(Value::Int(-value)),
                        Value::Float(value) => Ok(Value::Float(-value)),
                        _ => execution_error!("Invalid prefix operator: `{}`", op),
                    },
                    TokenKind::ExclamationMark => match value {
                        Value::Bool(value) => Ok(Value::Bool(!value)),
                        _ => execution_error!("Invalid prefix operator: `{}`", op),
                    },
                    _ => execution_error!("Invalid prefix operator: `{}`", op),
                }
            }
            _ => panic!("Expected prefix operator expression"),
        }
    }

    pub fn execute_infix_operator_expression(&mut self, infix_op: &Expr) -> BauResult<Value> {
        match &infix_op.kind {
            ExprKind::InfixOp { op, lhs, rhs } => {
                let lhs = self.execute_expression(lhs)?;
                let rhs = self.execute_expression(rhs)?;
                match op {
                    TokenKind::Plus => match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Int(lhs + rhs)),
                        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs + rhs)),
                        (Value::String(lhs), Value::String(rhs)) => {
                            Ok(Value::String(format!("{}{}", lhs, rhs)))
                        }
                        _ => execution_error!(
                            "Addition is only available between ints, floats and strings"
                        ),
                    },
                    TokenKind::Minus => match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Int(lhs - rhs)),
                        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs - rhs)),
                        _ => execution_error!(
                            "Subtraction is only available between ints and floats"
                        ),
                    },
                    TokenKind::Asterisk => match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Int(lhs * rhs)),
                        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs * rhs)),
                        _ => execution_error!(
                            "Multiplication is only available between ints and floats"
                        ),
                    },
                    TokenKind::Slash => match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Int(lhs / rhs)),
                        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Float(lhs / rhs)),
                        _ => execution_error!("Division is only available between ints and floats"),
                    },
                    TokenKind::EqualsEquals => Ok(Value::Bool(lhs == rhs)),
                    TokenKind::ExclamationMarkEquals => Ok(Value::Bool(lhs != rhs)),
                    TokenKind::LessThan => match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Bool(lhs < rhs)),
                        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs < rhs)),
                        _ => {
                            execution_error!("Less than is only available between ints and floats")
                        }
                    },
                    TokenKind::LessThanEquals => match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                        _ => execution_error!(
                            "Less than or equals is only available between ints and floats"
                        ),
                    },
                    TokenKind::GreaterThan => match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Bool(lhs > rhs)),
                        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs > rhs)),
                        _ => execution_error!(
                            "Greater than is only available between ints and floats"
                        ),
                    },
                    TokenKind::GreaterThanEquals => match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                        (Value::Float(lhs), Value::Float(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                        _ => execution_error!(
                            "Greater than or equals is only available between ints and floats"
                        ),
                    },
                    TokenKind::AmpersandAmpersand => match (lhs, rhs) {
                        (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs && rhs)),
                        _ => execution_error!("Logical and is only available between bools"),
                    },
                    TokenKind::PipePipe => match (lhs, rhs) {
                        (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs || rhs)),
                        _ => execution_error!("Logical or is only available between bools"),
                    },
                    _ => execution_error!("Invalid infix operator: `{}`", op),
                }
            }
            _ => panic!("Expected infix operator expression"),
        }
    }
}
