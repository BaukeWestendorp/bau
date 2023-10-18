use crate::parser::ast::Expr;

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
    Block {
        statements: Vec<Stmt>,
    },
    Return {
        expr: Option<Box<Expr>>,
    },
}
