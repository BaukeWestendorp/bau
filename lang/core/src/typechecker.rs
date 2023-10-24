use crate::builtins::BuiltinFunction;
use std::collections::HashMap;

use crate::error::BauResult;
use crate::parser::ast::{BlockKind, Literal};
use crate::parser::item::{ParsedExtendsItem, ParsedFunctionItem, ParsedItem};
use crate::parser::{ParsedExpr, ParsedExprKind, ParsedStmt, ParsedType};
use crate::tokenizer::token::{Span, TokenKind};
use crate::types::Type;

#[macro_export]
macro_rules! typechecker_error {
    ($span:expr, $($message:tt)*) => {
        Err(crate::error::BauError::TypecheckerError {
            span: $span,
            message: format!($($message)*),
        })
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum CheckedStmt {
    Let {
        name: String,
        var_type: TypeId,
        expr: Box<CheckedExpr>,
    },
    Assignment {
        name: String,
        expr: Box<CheckedExpr>,
    },
    If {
        condition: Box<CheckedExpr>,
        then_branch: Box<CheckedStmt>,
        else_branch: Option<Box<CheckedStmt>>,
    },
    Loop {
        body: Box<CheckedStmt>,
    },
    Block {
        block_kind: BlockKind,
        statements: Vec<CheckedStmt>,
    },
    Return {
        expr: Option<Box<CheckedExpr>>,
    },
    Continue,
    Break,
    Expression {
        expr: Box<CheckedExpr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckedExpr {
    kind: CheckedExprKind,
    type_id: TypeId,
    span: Span,
}

impl CheckedExpr {
    pub fn kind(&self) -> &CheckedExprKind {
        &self.kind
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CheckedExprKind {
    Literal(Literal),
    Identifier(String),
    BuiltinFnCall {
        function: BuiltinFunction,
        args: Vec<CheckedExpr>,
    },
    FnCall(CheckedFunctionCall),
    PrefixOp {
        op: TokenKind,
        expr: Box<CheckedExpr>,
    },
    InfixOp {
        op: TokenKind,
        lhs: Box<CheckedExpr>,
        rhs: Box<CheckedExpr>,
    },
    PostfixOp {
        op: TokenKind,
        expr: Box<CheckedExpr>,
    },
    MethodCall(CheckedFunctionItem),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckedFunctionCall {
    name: String,
    args: Vec<CheckedExpr>,
}

impl CheckedFunctionCall {
    pub fn new(name: String, args: Vec<CheckedExpr>) -> Self {
        Self { name, args }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn args(&self) -> &Vec<CheckedExpr> {
        &self.args
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckedFunctionItem {
    name: String,
    return_type: TypeId,
    parameters: Vec<(String, TypeId)>,
    body: CheckedStmt,
}

impl CheckedFunctionItem {
    pub fn new(
        name: &str,
        return_type: TypeId,
        parameters: Vec<(String, TypeId)>,
        body: CheckedStmt,
    ) -> Self {
        Self {
            name: name.to_string(),
            return_type,
            parameters,
            body,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn return_type(&self) -> TypeId {
        self.return_type
    }

    pub fn parameters(&self) -> &Vec<(String, TypeId)> {
        &self.parameters
    }

    pub fn body(&self) -> &CheckedStmt {
        &self.body
    }

    pub fn set_body(&mut self, body: CheckedStmt) {
        self.body = body;
    }
}

pub type TypeId = usize;
pub type FunctionId = usize;

pub const VOID_TYPE_ID: TypeId = 0;
pub const INT_TYPE_ID: TypeId = 1;
pub const FLOAT_TYPE_ID: TypeId = 2;
pub const STRING_TYPE_ID: TypeId = 3;
pub const BOOL_TYPE_ID: TypeId = 4;

pub struct Typechecker {
    variable_types: HashMap<String, TypeId>,
    functions: Vec<CheckedFunctionItem>,
    types: Vec<Type>,
}

impl Typechecker {
    pub fn new() -> Self {
        Self {
            variable_types: HashMap::new(),
            functions: vec![],
            types: vec![
                Type::new("void", vec![]),
                Type::new("int", vec![]),
                Type::new("float", vec![]),
                Type::new("string", vec![]),
                Type::new("bool", vec![]),
            ],
        }
    }

    /// Get the type for the given id.
    fn get_type(&self, type_id: TypeId) -> &Type {
        self.types
            .get(type_id)
            .expect(format!("Type with id `{}` not found", type_id).as_str())
    }

    /// Get the type for the given id.
    fn get_type_mut(&mut self, type_id: TypeId) -> &mut Type {
        self.types
            .get_mut(type_id)
            .expect(format!("Type with id `{}` not found", type_id).as_str())
    }

    /// Get the type of the variable with the given name.
    fn get_variable_type(&self, name: &str) -> TypeId {
        *self
            .variable_types
            .get(name)
            .expect(format!("Type not found for variable with name `{}`", name).as_str())
    }

    /// Set the type of the variable with the given name.
    fn set_variable_type(&mut self, name: String, type_id: TypeId) {
        self.variable_types.insert(name, type_id);
    }

    /// Get the function with the given name.
    fn get_function_by_name(&self, name: &str) -> BauResult<&CheckedFunctionItem> {
        let function = self
            .functions
            .iter()
            .find(|function| function.name() == name);
        match function {
            Some(function) => Ok(function),
            None => typechecker_error!(Span { start: 0, end: 0 }, "Function `{}` not found", name),
        }
    }

    pub fn functions(&self) -> &Vec<CheckedFunctionItem> {
        &self.functions
    }

    /// Set the function with the given name.
    fn set_function(&mut self, checked_function: CheckedFunctionItem) -> FunctionId {
        let function_id = self.functions.len();
        self.functions.push(checked_function);
        function_id
    }

    /// Get type id from a parsed type.
    fn id_from_parsed_type(&self, parsed_type: &ParsedType) -> TypeId {
        match parsed_type {
            ParsedType::Void => VOID_TYPE_ID,
            ParsedType::Int => INT_TYPE_ID,
            ParsedType::Float => FLOAT_TYPE_ID,
            ParsedType::String => STRING_TYPE_ID,
            ParsedType::Bool => BOOL_TYPE_ID,
            ParsedType::Name(name) => self
                .types
                .iter()
                .position(|type_| type_.name() == name)
                .expect(format!("Type with name `{}` not found", name).as_str()),
        }
    }

    /// Get the method on a type with the given name.
    fn get_method_mut(
        &mut self,
        type_id: TypeId,
        name: &str,
    ) -> BauResult<&mut CheckedFunctionItem> {
        let type_ = self.get_type(type_id).clone();
        let type_name = type_.name();
        let method = self
            .get_type_mut(type_id)
            .methods_mut()
            .iter_mut()
            .find(|method| method.name() == name);

        match method {
            Some(method) => Ok(method),
            None => typechecker_error!(
                // FIXME: Get span from method call
                Span { start: 0, end: 0 },
                "Method `{}` not found for type `{}`",
                name,
                type_name
            ),
        }
    }

    /// Add a method to a type as an extension.
    fn extend_type_with_method(
        &mut self,
        type_id: TypeId,
        method: CheckedFunctionItem,
    ) -> BauResult<()> {
        self.get_type_mut(type_id).add_method(method)
    }

    pub fn check_top_level(&mut self, top_level: &Vec<ParsedItem>) -> BauResult<()> {
        let mut extend_items = vec![];
        let mut function_items = vec![];
        top_level.iter().for_each(|item| match item {
            ParsedItem::Extends(extends_item) => extend_items.push(extends_item),
            ParsedItem::Function(function_item) => function_items.push(function_item),
        });
        // We have to check extends items first because we need to know
        // the methods on the types before we can check the function bodies.
        for extends_item in extend_items {
            self.check_extend_item(extends_item)?;
        }
        for function in function_items {
            self.check_function_item(function)?;
        }
        Ok(())
    }

    pub fn check_function_item(&mut self, function: &ParsedFunctionItem) -> BauResult<()> {
        let return_type = self.check_type(&function.return_type);
        let body = self.check_function_body(&function.body, return_type)?;
        self.set_function(CheckedFunctionItem::new(
            &function.name,
            return_type,
            vec![],
            body,
        ));
        Ok(())
    }

    pub fn check_extend_item(&mut self, extends_item: &ParsedExtendsItem) -> BauResult<()> {
        let type_id = self.check_type(&extends_item.parsed_type);
        for function in &extends_item.methods {
            let return_type = self.check_type(&function.return_type);
            let body = self.check_function_body(&function.body, return_type)?;
            self.extend_type_with_method(
                type_id,
                CheckedFunctionItem::new(&function.name, return_type, vec![], body),
            )?;
        }

        Ok(())
    }

    pub fn check_function_body(
        &mut self,
        body: &ParsedStmt,
        function_return_type: TypeId,
    ) -> BauResult<CheckedStmt> {
        match body {
            ParsedStmt::Block { statements, .. } => Ok(CheckedStmt::Block {
                block_kind: BlockKind::Function,
                statements: statements
                    .iter()
                    .map(|statement| self.check_statement(statement, function_return_type))
                    .collect::<BauResult<Vec<CheckedStmt>>>()?,
            }),
            _ => panic!("Function should have a block as body statement"),
        }
    }

    pub fn check_statement(
        &mut self,
        statement: &ParsedStmt,
        function_return_type: TypeId,
    ) -> BauResult<CheckedStmt> {
        match statement {
            ParsedStmt::Let { .. } => self.check_let_statement(statement),
            ParsedStmt::Assignment { .. } => self.check_assignment_statement(statement),
            ParsedStmt::If { .. } => self.check_if_statement(statement),
            ParsedStmt::Return { .. } => {
                self.check_return_statement(statement, function_return_type)
            }
            ParsedStmt::Expression { .. } => self.check_expression_statement(statement),
            _ => panic!("Statement not implemented: {:?}", statement),
        }
    }

    pub fn check_let_statement(&mut self, statement: &ParsedStmt) -> BauResult<CheckedStmt> {
        match statement {
            ParsedStmt::Let {
                parsed_type,
                expr,
                name,
            } => {
                let var_type_id = self.check_type(parsed_type);
                let expr = self.check_expression(expr)?;
                if var_type_id != expr.type_id {
                    return typechecker_error!(
                        expr.span,
                        "Type mismatch: expected `{}`, found `{}`",
                        self.get_type(var_type_id),
                        self.get_type(expr.type_id)
                    );
                }
                self.set_variable_type(name.clone(), var_type_id);
                Ok(CheckedStmt::Let {
                    name: name.clone(),
                    var_type: var_type_id,
                    expr: Box::new(expr),
                })
            }
            _ => panic!("Expected Let statement"),
        }
    }

    pub fn check_assignment_statement(&mut self, statement: &ParsedStmt) -> BauResult<CheckedStmt> {
        match statement {
            ParsedStmt::Assignment { expr, name } => {
                let expr = self.check_expression(expr)?;
                let var_type = self.get_variable_type(name);
                if var_type != expr.type_id {
                    return typechecker_error!(
                        expr.span,
                        "Type mismatch: expected `{}`, found `{}`",
                        self.get_type(var_type),
                        self.get_type(expr.type_id)
                    );
                }
                Ok(CheckedStmt::Assignment {
                    name: name.clone(),
                    expr: Box::new(expr),
                })
            }
            _ => panic!("Expected Assignment statement"),
        }
    }

    pub fn check_if_statement(&mut self, statement: &ParsedStmt) -> BauResult<CheckedStmt> {
        match statement {
            ParsedStmt::If { .. } => {
                todo!("Implement typechecking if statement")
            }
            _ => panic!("Expected If statement"),
        }
    }

    pub fn check_return_statement(
        &mut self,
        statement: &ParsedStmt,
        function_return_type: TypeId,
    ) -> BauResult<CheckedStmt> {
        match statement {
            ParsedStmt::Return { expr } => match expr {
                Some(expr) => {
                    let return_type = self.check_expression(expr)?.type_id;
                    if return_type != function_return_type {
                        return typechecker_error!(
                            expr.span,
                            "Expected `{}` return value, found `{}`",
                            self.get_type(function_return_type),
                            self.get_type(return_type)
                        );
                    }
                    Ok(CheckedStmt::Return {
                        expr: Some(Box::new(self.check_expression(expr)?)),
                    })
                }
                None => {
                    if function_return_type != VOID_TYPE_ID {
                        todo!("Implement error message")
                    }
                    Ok(CheckedStmt::Return { expr: None })
                }
            },
            _ => panic!("Expected Return statement"),
        }
    }

    pub fn check_expression_statement(&mut self, statement: &ParsedStmt) -> BauResult<CheckedStmt> {
        match statement {
            ParsedStmt::Expression { expr } => Ok(CheckedStmt::Expression {
                expr: Box::new(self.check_expression(expr)?),
            }),
            _ => panic!("Expected Expression statement"),
        }
    }

    pub fn check_expression(&mut self, expression: &ParsedExpr) -> BauResult<CheckedExpr> {
        let expr = match &expression.kind {
            ParsedExprKind::Literal(literal) => CheckedExpr {
                kind: CheckedExprKind::Literal(literal.clone()),
                type_id: self.get_type_from_literal(literal),
                span: expression.span,
            },
            ParsedExprKind::Identifier(identifier) => CheckedExpr {
                kind: CheckedExprKind::Identifier(identifier.clone()),
                type_id: self.get_variable_type(identifier),
                span: expression.span,
            },
            ParsedExprKind::BuiltinFnCall { function, args } => CheckedExpr {
                kind: CheckedExprKind::BuiltinFnCall {
                    function: function.clone(),
                    args: args
                        .iter()
                        .map(|arg| self.check_expression(arg))
                        .collect::<BauResult<Vec<CheckedExpr>>>()?,
                },
                type_id: function.function.return_type,
                span: expression.span,
            },
            ParsedExprKind::FnCall(call) => {
                let expr_type = self.get_type_from_function_call(expression)?;
                CheckedExpr {
                    kind: CheckedExprKind::FnCall(CheckedFunctionCall {
                        name: call.name.clone(),
                        args: vec![],
                    }),
                    type_id: expr_type,
                    span: expression.span,
                }
            }
            ParsedExprKind::PrefixOp { .. } => todo!("Getting type from PrefixOp not implemented"),
            ParsedExprKind::InfixOp { lhs, op, rhs } => {
                let lhs = self.check_expression(lhs)?;
                let rhs = self.check_expression(rhs)?;
                if lhs.type_id != rhs.type_id {
                    return typechecker_error!(
                        rhs.span,
                        "Type mismatch: expected `{}`, found `{}`",
                        self.get_type(lhs.type_id),
                        self.get_type(rhs.type_id)
                    );
                }
                CheckedExpr {
                    kind: CheckedExprKind::InfixOp {
                        op: op.clone(),
                        lhs: Box::new(lhs.clone()),
                        rhs: Box::new(rhs),
                    },
                    type_id: lhs.type_id,
                    span: expression.span,
                }
            }
            ParsedExprKind::PostfixOp { .. } => {
                todo!("Getting type from PostfixOp not implemented")
            }
            ParsedExprKind::MethodCall { expr, call } => {
                let checked_expr = self.check_expression(expr)?;
                let method = self.get_method_mut(checked_expr.type_id, &call.name)?;
                CheckedExpr {
                    kind: CheckedExprKind::MethodCall(method.clone()),
                    span: expression.span,
                    type_id: method.return_type,
                }
            }
        };

        Ok(expr)
    }

    pub fn check_type(&mut self, parsed_type: &ParsedType) -> TypeId {
        self.id_from_parsed_type(parsed_type)
    }

    pub fn get_type_from_literal(&mut self, literal: &Literal) -> TypeId {
        match literal {
            Literal::Int(_) => INT_TYPE_ID,
            Literal::Float(_) => FLOAT_TYPE_ID,
            Literal::String(_) => STRING_TYPE_ID,
            Literal::Bool(_) => BOOL_TYPE_ID,
        }
    }

    pub fn get_type_from_function_call(&self, expression: &ParsedExpr) -> BauResult<TypeId> {
        match &expression.kind {
            ParsedExprKind::FnCall(call) => Ok(self.get_function_by_name(&call.name)?.return_type),
            _ => panic!("Expected FnCall expression"),
        }
    }
}
