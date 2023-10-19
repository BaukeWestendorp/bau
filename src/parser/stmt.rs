use crate::error::BauResult;
use crate::parser::ast::Stmt;
use crate::parser::Parser;
use crate::tokenizer::token::TokenKind;

impl Parser<'_> {
    pub fn parse_statement(&mut self) -> BauResult<Stmt> {
        match self.peek_kind() {
            TokenKind::Let => self.parse_let_statement(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::Loop => self.parse_loop_statement(),
            TokenKind::BraceOpen => self.parse_block_statement(),
            TokenKind::Identifier => {
                let next = self.peek_offset_kind(1);
                match next {
                    TokenKind::Equals => self.parse_assignment_statement(),
                    _ => self.parse_expression_statement(),
                }
            }
            _ => self.parse_expression_statement(),
        }
    }

    pub fn parse_let_statement(&mut self) -> BauResult<Stmt> {
        self.consume_specific(TokenKind::Let)?;
        let ident = self.consume_specific(TokenKind::Identifier)?;
        let name = self.text(ident).to_string();
        self.consume_specific(TokenKind::Equals)?;
        let value = self.parse_expression()?;
        self.consume_specific(TokenKind::Semicolon)?;
        Ok(Stmt::Let {
            name,
            expr: Box::new(value),
        })
    }

    pub fn parse_if_statement(&mut self) -> BauResult<Stmt> {
        self.consume_specific(TokenKind::If)?;
        let condition = self.parse_expression()?;
        let then_branch = self.parse_block_statement()?;
        let else_branch = if self.at(TokenKind::Else) {
            self.consume_specific(TokenKind::Else)?;
            Some(Box::new(self.parse_block_statement()?))
        } else {
            None
        };
        Ok(Stmt::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    pub fn parse_return_statement(&mut self) -> BauResult<Stmt> {
        self.consume_specific(TokenKind::Return)?;

        // No return value
        if self.at(TokenKind::Semicolon) {
            self.consume_specific(TokenKind::Semicolon)?;
            return Ok(Stmt::Return { expr: None });
        }

        let value = self.parse_expression()?;
        self.consume_specific(TokenKind::Semicolon)?;
        Ok(Stmt::Return {
            expr: Some(Box::new(value)),
        })
    }

    pub fn parse_loop_statement(&mut self) -> BauResult<Stmt> {
        self.consume_specific(TokenKind::Loop)?;

        let body = self.parse_block_statement()?;
        Ok(Stmt::Loop {
            body: Box::new(body),
        })
    }

    pub fn parse_block_statement(&mut self) -> BauResult<Stmt> {
        self.consume_specific(TokenKind::BraceOpen)?;
        let mut statements = vec![];
        while !self.at(TokenKind::BraceClose) {
            let statement = self.parse_statement()?;
            statements.push(statement);
        }
        self.consume_specific(TokenKind::BraceClose)?;
        Ok(Stmt::Block { statements })
    }

    pub fn parse_assignment_statement(&mut self) -> BauResult<Stmt> {
        let ident = self.consume_specific(TokenKind::Identifier)?;
        let name = self.text(ident).to_string();
        self.consume_specific(TokenKind::Equals)?;
        let value = self.parse_expression()?;
        self.consume_specific(TokenKind::Semicolon)?;
        Ok(Stmt::Assignment {
            name,
            expr: Box::new(value),
        })
    }

    pub fn parse_expression_statement(&mut self) -> BauResult<Stmt> {
        let expr = self.parse_expression()?;
        self.consume_specific(TokenKind::Semicolon)?;
        Ok(Stmt::Expression {
            expr: Box::new(expr),
        })
    }
}
