use crate::error::BauResult;
use crate::execution_error;
use crate::interpreter::value::Value;
use crate::interpreter::Interpreter;
use crate::parser::ast::{Expr, Item, Literal, Stmt};
use crate::tokenizer::token::TokenKind;

impl Interpreter {
    pub fn execute_main(&mut self) -> BauResult<()> {
        match self.main_function()?.clone() {
            main @ Item::Function { .. } => {
                self.execute_function(&main, &vec![])?;
            }
        }
        Ok(())
    }

    pub fn execute_function(&mut self, function: &Item, _args: &Vec<Expr>) -> BauResult<Value> {
        match function {
            Item::Function { body, .. } => self.execute_block_statement(body),
        }
    }

    pub fn execute_statement(&mut self, statement: &Stmt) -> BauResult<Value> {
        match statement {
            Stmt::Return { .. } => self.execute_return_statement(statement),
            Stmt::Let { .. } => self.execute_let_statement(statement),
            Stmt::Assignment { .. } => self.execute_assignment_statement(statement),
            Stmt::If { .. } => execution_error!("If statement not implemented"),
            Stmt::Block { .. } => self.execute_block_statement(statement),
            Stmt::Loop { .. } => self.execute_loop_statement(statement),
            Stmt::Expression { .. } => self.execute_expression_statement(statement),
        }
    }

    pub fn execute_return_statement(&mut self, statement: &Stmt) -> BauResult<Value> {
        match statement {
            Stmt::Return { expr } => {
                let value = match expr {
                    Some(value) => value,
                    None => return Ok(Value::none()),
                };
                let value = self.execute_expression(value)?;
                Ok(value)
            }
            _ => execution_error!("Expected return statement"),
        }
    }

    pub fn execute_let_statement(&mut self, statement: &Stmt) -> BauResult<Value> {
        match statement {
            Stmt::Let { name, expr } => {
                let initial_value = self.execute_expression(expr)?;
                self.variables.insert(name.clone(), initial_value);
                Ok(Value::none())
            }
            _ => execution_error!("Expected let statement"),
        }
    }

    pub fn execute_assignment_statement(&mut self, statement: &Stmt) -> BauResult<Value> {
        match statement {
            Stmt::Assignment { name, expr } => {
                let value = self.execute_expression(expr)?;
                if !self.variables.contains_key(name) {
                    return execution_error!("No variable found with name: `{}`", name);
                }
                self.variables.insert(name.clone(), value);
                Ok(Value::none())
            }
            _ => execution_error!("Expected assignment statement"),
        }
    }

    pub fn execute_block_statement(&mut self, statement: &Stmt) -> BauResult<Value> {
        match statement {
            Stmt::Block { statements } => {
                let mut last_result = Ok(Value::none());
                for statement in statements {
                    last_result = self.execute_statement(statement);
                    if last_result.is_err() {
                        return last_result;
                    }
                }
                last_result
            }
            _ => execution_error!("Expected block statement"),
        }
    }

    pub fn execute_loop_statement(&mut self, statement: &Stmt) -> BauResult<Value> {
        match statement {
            Stmt::Loop { body } => loop {
                let result = self.execute_statement(body);
                if result.is_err() {
                    return result;
                }
            },
            _ => execution_error!("Expected loop statement"),
        }
    }

    pub fn execute_expression_statement(&mut self, expression: &Stmt) -> BauResult<Value> {
        match expression {
            Stmt::Expression { expr } => self.execute_expression(expr),
            _ => execution_error!("Expected expression statement"),
        }
    }

    pub fn execute_expression(&mut self, expression: &Expr) -> BauResult<Value> {
        match expression {
            Expr::Literal(literal) => self.execute_literal_expression(literal),
            Expr::Ident(ident) => self.execute_ident_expression(ident),
            Expr::FnCall { .. } => self.execute_function_call_expression(expression),
            Expr::PrefixOp { .. } => self.execute_prefix_operator_expression(expression),
            Expr::InfixOp { .. } => {
                execution_error!("InfixOp expression execution not implemented")
            }
            Expr::PostfixOp { .. } => {
                execution_error!("PostfixOp expression execution not implemented")
            }
            Expr::BuiltinFnCall { function, args } => function.call(self, args),
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

    pub fn execute_ident_expression(&mut self, ident: &str) -> BauResult<Value> {
        match self.variables.get(ident) {
            Some(value) => Ok(value.clone()),
            None => execution_error!("No variable found with name: `{}`", ident),
        }
    }

    pub fn execute_function_call_expression(&mut self, function_call: &Expr) -> BauResult<Value> {
        match function_call {
            Expr::FnCall { name, args } => {
                let function = match self.functions.get(name) {
                    Some(function) => function.clone(),
                    None => return execution_error!("No function found with name: `{}`", name),
                };

                let value = self.execute_function(&function, args)?;
                return Ok(value);
            }
            _ => execution_error!("Expected function call expression"),
        }
    }

    pub fn execute_prefix_operator_expression(&mut self, prefix_op: &Expr) -> BauResult<Value> {
        match prefix_op {
            Expr::PrefixOp { op, expr } => {
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
            _ => execution_error!("Expected prefix operator expression"),
        }
    }
}
