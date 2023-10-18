use crate::error::{BauError, BauResult};
use crate::interpreter::value::Value;
use crate::interpreter::Interpreter;
use crate::interpreter::PRINT_FUNCTION_NAME;
use crate::parser::ast::{Expr, Item, Literal, Stmt};

impl Interpreter {
    pub fn execute_main(&mut self) -> BauResult<()> {
        match self.main_function()?.clone() {
            main @ Item::Function { .. } => {
                self.execute_function(&main, &vec![])?;
            }
        }
        Ok(())
    }

    pub fn execute_function(
        &mut self,
        function: &Item,
        args: &Vec<Expr>,
    ) -> BauResult<Option<Value>> {
        match function {
            Item::Function { name, body, .. } => {
                match name.as_str() {
                    PRINT_FUNCTION_NAME => {
                        let value = match self.execute_expression(&args[0])? {
                            Some(value) => value,
                            None => {
                                return Err(BauError::ExecutionError {
                                    message: "Expected value to print".to_string(),
                                })
                            }
                        };
                        println!("{:?}", value);
                        return Ok(None);
                    }
                    _ => {}
                }

                let mut last_result = Ok(None);
                for statement in body {
                    last_result = self.execute_statement(statement);
                    if last_result.is_err() {
                        return last_result;
                    }
                }
                last_result
            }
        }
    }

    pub fn execute_statement(&mut self, statement: &Stmt) -> BauResult<Option<Value>> {
        match statement {
            Stmt::Return { .. } => self.execute_return_statement(statement),
            Stmt::Let { .. } => todo!("Implement Let statement expression"),
            Stmt::Assignment { .. } => todo!("Implement Assignment statement expression"),
            Stmt::If { .. } => todo!("Implement If statement expression"),
            Stmt::Block { .. } => todo!("Implement Block statement expression"),
            Stmt::Expression { .. } => self.execute_expression_statement(statement),
        }
    }

    pub fn execute_return_statement(&mut self, statement: &Stmt) -> BauResult<Option<Value>> {
        match statement {
            Stmt::Return { expr } => {
                let value = match expr {
                    Some(value) => value,
                    None => return Ok(None),
                };
                let value = self.execute_expression(value)?;
                Ok(value)
            }
            _ => Err(BauError::ExecutionError {
                message: "Expected return statement".to_string(),
            }),
        }
    }

    pub fn execute_expression_statement(&mut self, expression: &Stmt) -> BauResult<Option<Value>> {
        match expression {
            Stmt::Expression { expr } => self.execute_expression(expr),
            _ => Err(BauError::ExecutionError {
                message: "Expected expression statement".to_string(),
            }),
        }
    }

    pub fn execute_expression(&mut self, expression: &Expr) -> BauResult<Option<Value>> {
        match expression {
            Expr::Literal(literal) => self.execute_literal_expression(literal).map(Some),
            Expr::Ident(_) => todo!("Implement Ident expression execution"),
            Expr::FnCall { .. } => self.execute_function_call_expression(expression),
            Expr::PrefixOp { .. } => todo!("Implement PrefixOp expression execution"),
            Expr::InfixOp { .. } => todo!("Implement InfixOp expression execution"),
            Expr::PostfixOp { .. } => todo!("Implement PostfixOp expression execution"),
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

    pub fn execute_function_call_expression(
        &mut self,
        function_call: &Expr,
    ) -> BauResult<Option<Value>> {
        match function_call {
            Expr::FnCall { name, args } => {
                let function = match self.functions.get(name) {
                    Some(function) => function.clone(),
                    None => {
                        return Err(BauError::ExecutionError {
                            message: format!("No function found with name: {}", name),
                        })
                    }
                };

                let value = self.execute_function(&function, args)?;
                return Ok(value);
            }
            _ => Err(BauError::ExecutionError {
                message: "Expected function call expression".to_string(),
            }),
        }
    }
}
