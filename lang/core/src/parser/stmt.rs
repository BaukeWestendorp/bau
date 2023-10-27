use crate::error::BauResult;
use crate::parser::ast::BlockKind;
use crate::parser::{ParsedStmt, ParsedType, Parser};
use crate::tokenizer::token::TokenKind;

impl Parser<'_> {
    pub fn parse_statement(&mut self) -> BauResult<ParsedStmt> {
        match self.peek_kind() {
            TokenKind::Let => self.parse_let_statement(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::Loop => self.parse_loop_statement(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::Continue => self.parse_continue_statement(),
            TokenKind::Break => self.parse_break_statement(),
            TokenKind::BraceOpen => self.parse_block_statement(BlockKind::Regular),
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

    pub fn parse_let_statement(&mut self) -> BauResult<ParsedStmt> {
        self.consume_specific(TokenKind::Let)?;
        let var_type = self.parse_type()?;
        let name_ident = self.consume_specific(TokenKind::Identifier)?;
        let name = self.text(name_ident).to_string();
        self.consume_specific(TokenKind::Equals)?;
        let value = self.parse_expression()?;
        self.consume_specific(TokenKind::Semicolon)?;
        Ok(ParsedStmt::Let {
            name,
            parsed_type: var_type,
            expr: value,
        })
    }

    pub fn parse_if_statement(&mut self) -> BauResult<ParsedStmt> {
        self.consume_specific(TokenKind::If)?;
        let condition = self.parse_expression()?;
        let then_branch = self.parse_block_statement(BlockKind::Regular)?;
        let else_branch = if self.at(TokenKind::Else) {
            self.consume_specific(TokenKind::Else)?;
            Some(Box::new(self.parse_block_statement(BlockKind::Regular)?))
        } else {
            None
        };
        Ok(ParsedStmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    pub fn parse_loop_statement(&mut self) -> BauResult<ParsedStmt> {
        self.consume_specific(TokenKind::Loop)?;

        let body = self.parse_block_statement(BlockKind::Loop)?;
        Ok(ParsedStmt::Loop {
            body: Box::new(body),
        })
    }

    pub fn parse_return_statement(&mut self) -> BauResult<ParsedStmt> {
        self.consume_specific(TokenKind::Return)?;

        // No return value
        if self.at(TokenKind::Semicolon) {
            self.consume_specific(TokenKind::Semicolon)?;
            return Ok(ParsedStmt::Return { expr: None });
        }

        let value = self.parse_expression()?;
        self.consume_specific(TokenKind::Semicolon)?;
        Ok(ParsedStmt::Return { expr: Some(value) })
    }

    pub fn parse_continue_statement(&mut self) -> BauResult<ParsedStmt> {
        self.consume_specific(TokenKind::Continue)?;
        self.consume_specific(TokenKind::Semicolon)?;
        Ok(ParsedStmt::Continue)
    }

    pub fn parse_break_statement(&mut self) -> BauResult<ParsedStmt> {
        self.consume_specific(TokenKind::Break)?;
        self.consume_specific(TokenKind::Semicolon)?;
        Ok(ParsedStmt::Break)
    }

    pub fn parse_block_statement(&mut self, block_kind: BlockKind) -> BauResult<ParsedStmt> {
        self.consume_specific(TokenKind::BraceOpen)?;
        let mut statements = vec![];
        while !self.at(TokenKind::BraceClose) {
            let statement = self.parse_statement()?;
            statements.push(statement);
        }
        self.consume_specific(TokenKind::BraceClose)?;
        Ok(ParsedStmt::Block {
            statements,
            block_kind,
        })
    }

    pub fn parse_assignment_statement(&mut self) -> BauResult<ParsedStmt> {
        let ident = self.consume_specific(TokenKind::Identifier)?;
        let name = self.text(ident).to_string();
        self.consume_specific(TokenKind::Equals)?;
        let value = self.parse_expression()?;
        self.consume_specific(TokenKind::Semicolon)?;
        Ok(ParsedStmt::Assignment { name, expr: value })
    }

    pub fn parse_expression_statement(&mut self) -> BauResult<ParsedStmt> {
        let expr = self.parse_expression()?;
        self.consume_specific(TokenKind::Semicolon)?;
        Ok(ParsedStmt::Expression { expr })
    }

    pub fn parse_type(&mut self) -> BauResult<ParsedType> {
        let ident = self.consume_specific(TokenKind::Identifier)?;
        let type_name = self.text(ident);
        Ok(ParsedType::Name(type_name.to_string()))
    }
}