use crate::parser::stmt::Stmt;
use crate::tokenizer::token::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Ident(String),
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
        parameters: Vec<(String, Type)>,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    pub name: String,
}
