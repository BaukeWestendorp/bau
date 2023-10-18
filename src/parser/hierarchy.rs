use crate::error::BauResult;
use crate::parser::ast::{Item, Type};
use crate::parser::stmt::Stmt;
use crate::parser::Parser;
use crate::tokenizer::token::{Token, TokenKind};

impl<'input, I> Parser<'input, I>
where
    I: Iterator<Item = Token>,
{
    pub fn parse_top_level(&mut self) -> BauResult<Vec<Item>> {
        let mut items = vec![];
        while !self.at(TokenKind::EndOfFile) {
            let item = self.parse_item()?;
            items.push(item)
        }
        Ok(items)
    }

    pub fn parse_type(&mut self) -> BauResult<Type> {
        let ident = match self.next() {
            Some(ident) => ident,
            None => return Err(self.error("Unexpected EOF while parsing type".to_string())),
        };
        assert_eq!(
            ident.kind,
            TokenKind::Identifier,
            "Expected identifier at start of type, but found `{:?}`",
            ident.kind
        );
        let name = self.text(ident).to_string();
        Ok(Type { name })
    }

    pub fn parse_item(&mut self) -> BauResult<Item> {
        match self.peek() {
            TokenKind::Fn => {
                self.consume(TokenKind::Fn)?;
                let ident = self.next().expect("Expected identifier after `fn`");
                assert_eq!(
                    ident.kind,
                    TokenKind::Identifier,
                    "Expected identifier after `fn`, but found `{:?}`",
                    ident.kind
                );
                let name = self.text(ident).to_string();

                self.consume(TokenKind::ParenOpen)?;
                // FIXME: Parse arguments
                self.consume(TokenKind::ParenClose)?;
                assert!(
                    self.at(TokenKind::BraceOpen),
                    "Expected `{{` after function declaration"
                );
                let body = match self.parse_statement()? {
                    Stmt::Block { statements } => statements,
                    _ => unreachable!(),
                };

                Ok(Item::Function {
                    name,
                    parameters: vec![],
                    body,
                })
            }

            unknown => Err(self.error(format!("Unexpected token: `{:?}`", unknown))),
        }
    }

    pub fn parse_statement(&mut self) -> BauResult<Stmt> {
        match self.peek() {
            TokenKind::Let => self.parse_let_statement(),
            TokenKind::If => todo!("Implement if statement"),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::BraceOpen => self.parse_block_statement(),
            TokenKind::Identifier => self.parse_identifier_statement(),
            unknown => Err(self.error(format!(
                "Unexpected token while parsing statement: `{unknown:?}`"
            ))),
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

    pub fn parse_identifier_statement(&mut self) -> BauResult<Stmt> {
        let ident = self.next().unwrap();
        let name = self.text(ident).to_string();
        self.consume(TokenKind::Equals)?;
        let value = self.parse_expression()?;
        self.consume(TokenKind::Semicolon)?;
        Ok(Stmt::Assignment {
            name,
            expr: Box::new(value),
        })
    }
}
