use crate::parser::ast::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        value: Box<Expr>,
    },
    Assignment {
        name: String,
        value: Box<Expr>,
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
        value: Box<Expr>,
    },
}
