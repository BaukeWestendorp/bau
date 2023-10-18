use crate::error::BauResult;
use crate::parser::ast::{Item, Stmt};
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

    pub fn parse_item(&mut self) -> BauResult<Item> {
        match self.peek() {
            TokenKind::Fn => self.parse_function_item(),
            unknown => Err(self.error(format!("Unexpected token: `{:?}`", unknown))),
        }
    }

    pub fn parse_function_item(&mut self) -> BauResult<Item> {
        self.consume(TokenKind::Fn)?;
        let ident = self.consume(TokenKind::Identifier)?;
        let name = self.text(ident).to_string();

        self.consume(TokenKind::ParenOpen)?;
        let mut parameters = vec![];
        while !self.at(TokenKind::ParenClose) {
            let param_ident = self.consume(TokenKind::Identifier)?;
            let name = self.text(param_ident).to_string();
            parameters.push(name);
        }
        self.consume(TokenKind::ParenClose)?;
        if !self.at(TokenKind::BraceOpen) {
            return Err(self.error("Expected `{` after function declaration".to_string()));
        }
        let body = match self.parse_statement()? {
            Stmt::Block { statements } => statements,
            _ => unreachable!(),
        };

        Ok(Item::Function {
            name,
            parameters,
            body,
        })
    }
}
