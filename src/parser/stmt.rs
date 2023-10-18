use crate::error::BauResult;
use crate::parser::ast::Stmt;
use crate::parser::Parser;
use crate::tokenizer::token::{Token, TokenKind};

impl<'input, I> Parser<'input, I>
where
    I: Iterator<Item = Token>,
{
    pub fn parse_statement(&mut self) -> BauResult<Stmt> {
        match self.peek() {
            TokenKind::Let => self.parse_let_statement(),
            TokenKind::If => todo!("Implement if statement"),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::BraceOpen => self.parse_block_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    pub fn parse_let_statement(&mut self) -> BauResult<Stmt> {
        self.consume(TokenKind::Let)?;
        let ident = self.next().expect("Expected identifier after `let`");
        assert_eq!(
            ident.kind,
            TokenKind::Identifier,
            "Expected identifier after `let`, but found `{:?}`",
            ident.kind
        );
        let name = self.text(ident).to_string();
        self.consume(TokenKind::Equals)?;
        let value = self.parse_expression()?;
        self.consume(TokenKind::Semicolon)?;
        Ok(Stmt::Let {
            name,
            expr: Box::new(value),
        })
    }

    pub fn parse_return_statement(&mut self) -> BauResult<Stmt> {
        self.consume(TokenKind::Return)?;

        // No return value
        if self.at(TokenKind::Semicolon) {
            self.consume(TokenKind::Semicolon)?;
            return Ok(Stmt::Return { expr: None });
        }

        let value = self.parse_expression()?;
        self.consume(TokenKind::Semicolon)?;
        Ok(Stmt::Return {
            expr: Some(Box::new(value)),
        })
    }

    pub fn parse_block_statement(&mut self) -> BauResult<Stmt> {
        self.consume(TokenKind::BraceOpen)?;
        let mut statements = vec![];
        while !self.at(TokenKind::BraceClose) {
            let statement = self.parse_statement()?;
            statements.push(statement);
        }
        self.consume(TokenKind::BraceClose)?;
        Ok(Stmt::Block { statements })
    }

    pub fn parse_expression_statement(&mut self) -> BauResult<Stmt> {
        let expr = self.parse_expression()?;
        self.consume(TokenKind::Semicolon)?;
        Ok(Stmt::Expression {
            expr: Box::new(expr),
        })
    }
}
