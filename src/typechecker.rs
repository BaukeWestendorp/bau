use std::collections::HashMap;

use crate::error::BauResult;
use crate::parser::ast::{Expr, ExprKind, Item, Literal, Stmt, Type};

macro_rules! typechecker_error {
    ($span:expr, $($message:tt)*) => {
        Err(crate::error::BauError::TypecheckerError {
            span: $span,
            message: format!($($message)*),
        })
    };
}

pub struct Typechecker {
    types: HashMap<String, Type>,
}

impl Typechecker {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }

    pub fn get_type(&self, variable_name: String) -> &Type {
        self.types
            .get(&variable_name)
            .expect(format!("Type not found for variable with name `{}`", variable_name).as_str())
    }

    pub fn set_type(&mut self, variable_name: String, var_type: Type) {
        self.types.insert(variable_name, var_type);
    }

    pub fn check_top_level(&mut self, top_level: &Vec<Item>) -> BauResult<()> {
        let mut last_result = Ok(());
        for item in top_level {
            last_result = match item {
                function @ Item::Function { .. } => self.check_function_item(function),
            };

            if last_result.is_err() {
                return last_result;
            }
        }
        last_result
    }

    pub fn check_function_item(&mut self, function: &Item) -> BauResult<()> {
        match &function {
            Item::Function { body, .. } => match body {
                Stmt::Block { statements, .. } => {
                    for statement in statements {
                        self.check_statement(statement)?;
                    }
                    Ok(())
                }
                _ => panic!("Function should have a block as body statement"),
            },
        }
    }

    pub fn check_statement(&mut self, statement: &Stmt) -> BauResult<()> {
        match statement {
            Stmt::Let { .. } => self.check_let_statement(statement),
            Stmt::Assignment { .. } => self.check_assignment_statement(statement),
            Stmt::If { .. } => self.check_if_statement(statement),
            Stmt::Loop { .. } => todo!("Typechecking Loop statement not implemented"),
            Stmt::Block { .. } => todo!("Typechecking Block statement not implemented"),
            Stmt::Return { .. } => todo!("Typechecking Return statement not implemented"),
            Stmt::Continue => todo!("Typechecking Continue statement not implemented"),
            Stmt::Break => todo!("Typechecking Break statement not implemented"),
            Stmt::Expression { .. } => self.check_expression_statement(statement),
        }
    }

    pub fn check_let_statement(&mut self, statement: &Stmt) -> BauResult<()> {
        match statement {
            Stmt::Let {
                var_type,
                expr,
                name,
            } => {
                let expr_type = self.get_type_from_expression(expr)?;
                if var_type != &expr_type {
                    return typechecker_error!(
                        expr.span,
                        "Type mismatch: expected `{}`, found `{}`",
                        var_type,
                        expr_type
                    );
                }

                self.set_type(name.clone(), var_type.clone());
                Ok(())
            }
            _ => panic!("Expected Let statement"),
        }
    }

    pub fn check_assignment_statement(&self, statement: &Stmt) -> BauResult<()> {
        match statement {
            Stmt::Assignment { expr, name } => {
                let expr_type = self.get_type_from_expression(expr)?;
                let var_type = self.get_type(name.clone());
                if var_type != &expr_type {
                    return typechecker_error!(
                        expr.span,
                        "Type mismatch: expected `{}`, found `{}`",
                        var_type,
                        expr_type
                    );
                }
                Ok(())
            }
            _ => panic!("Expected Assignment statement"),
        }
    }

    pub fn check_if_statement(&self, statement: &Stmt) -> BauResult<()> {
        match statement {
            Stmt::If { condition, .. } => {
                let condition_type = self.get_type_from_expression(condition)?;
                if condition_type != Type::bool() {
                    return typechecker_error!(
                        condition.span,
                        "The condition of an if statement should express a boolean value. Found `{}`",
                        condition_type
                    );
                }
                Ok(())
            }
            _ => panic!("Expected If statement"),
        }
    }

    pub fn check_expression_statement(&self, stmt: &Stmt) -> BauResult<()> {
        match stmt {
            Stmt::Expression { expr } => {
                self.get_type_from_expression(expr)?;
                Ok(())
            }
            _ => panic!("Expected Expression statement"),
        }
    }

    pub fn get_type_from_expression(&self, expr: &Expr) -> BauResult<Type> {
        match &expr.kind {
            ExprKind::Literal(literal) => Ok(self.get_type_from_literal(literal)),
            ExprKind::Identifier(_) => todo!("Getting type from Identifier not implemented"),
            ExprKind::BuiltinFnCall { .. } => Ok(Type::unit()),
            ExprKind::FnCall { .. } => todo!("Getting type from FnCall not implemented"),
            ExprKind::PrefixOp { .. } => todo!("Getting type from PrefixOp not implemented"),
            ExprKind::InfixOp { .. } => self.get_type_from_infix_operator(expr),
            ExprKind::PostfixOp { .. } => todo!("Getting type from PostfixOp not implemented"),
        }
    }

    pub fn get_type_from_literal(&self, literal: &Literal) -> Type {
        match literal {
            Literal::Int(_) => Type::int(),
            Literal::Float(_) => Type::float(),
            Literal::String(_) => Type::string(),
            Literal::Bool(_) => Type::bool(),
        }
    }

    pub fn get_type_from_infix_operator(&self, infix_op: &Expr) -> BauResult<Type> {
        match &infix_op.kind {
            ExprKind::InfixOp { lhs, rhs, .. } => {
                let lhs_type = self.get_type_from_expression(&lhs)?;
                let rhs_type = self.get_type_from_expression(&rhs)?;
                if lhs_type != rhs_type {
                    return typechecker_error!(
                        infix_op.span,
                        "Type mismatch: expected `{}`, found `{}`",
                        lhs_type,
                        rhs_type
                    );
                }
                Ok(Type::bool())
            }
            _ => panic!("Expected InfixOp expression"),
        }
    }
}
