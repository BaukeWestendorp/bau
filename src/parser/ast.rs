use crate::builtins::BuiltinFunction;
use crate::tokenizer::token::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    BuiltinFnCall {
        function: BuiltinFunction,
        args: Vec<Expr>,
    },
    FnCall {
        name: String,
        args: Vec<Expr>,
    },
    PrefixOp {
        op: TokenKind,
        expr: Box<Expr>,
    },
    InfixOp {
        op: TokenKind,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    PostfixOp {
        op: TokenKind,
        expr: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        expr: Box<Expr>,
    },
    Assignment {
        name: String,
        expr: Box<Expr>,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Loop {
        body: Box<Stmt>,
    },
    Block {
        statements: Vec<Stmt>,
    },
    Return {
        expr: Option<Box<Expr>>,
    },
    Continue,
    Break,
    Expression {
        expr: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Function {
        name: String,
        parameters: Vec<String>,
        body: Stmt,
    },
}
