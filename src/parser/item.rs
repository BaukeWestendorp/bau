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
        let ident = self.next().expect("Expected identifier after `fn`");
        assert_eq!(
            ident.kind,
            TokenKind::Identifier,
            "Expected identifier after `fn`, but found `{:?}`",
            ident.kind
        );
        let name = self.text(ident).to_string();

        self.consume(TokenKind::ParenOpen)?;
        let mut parameters = vec![];
        while !self.at(TokenKind::ParenClose) {
            let param = self.next().expect("Expected identifier as parameter");
            assert_eq!(
                param.kind,
                TokenKind::Identifier,
                "Expected identifier as parameter, but found `{:?}`",
                param.kind
            );
            let name = self.text(param).to_string();
            parameters.push(name);
        }
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
            parameters,
            body,
        })
    }
}
