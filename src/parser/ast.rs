use crate::builtins::BuiltinFunction;
use crate::tokenizer::token::{Span, TokenKind};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
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
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    PostfixOp {
        op: TokenKind,
        expr: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockKind {
    Regular,
    Loop,
    Function,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        var_type: Type,
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
        block_kind: BlockKind,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub name: String,
}
impl Type {
    pub fn unit() -> Type {
        Type {
            name: "()".to_string(),
        }
    }
    pub fn int() -> Type {
        Type {
            name: "int".to_string(),
        }
    }

    pub fn float() -> Type {
        Type {
            name: "float".to_string(),
        }
    }

    pub fn string() -> Type {
        Type {
            name: "string".to_string(),
        }
    }

    pub fn bool() -> Type {
        Type {
            name: "bool".to_string(),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
