use crate::error::{BauResult, ParserError};
use crate::parser::{ParsedStmt, ParsedType, Parser};
use crate::tokenizer::token::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedItem {
    Function(ParsedFunctionItem),
    Extends(ParsedExtendsItem),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFunctionItem {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: ParsedStmt,
    pub return_type: ParsedType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedExtendsItem {
    pub parsed_type: ParsedType,
    pub methods: Vec<ParsedFunctionItem>,
}

impl Parser<'_> {
    pub fn parse_top_level(&mut self) -> BauResult<Vec<ParsedItem>> {
        let mut items = vec![];
        while !self.at(TokenKind::EndOfFile) {
            let item = self.parse_item()?;
            items.push(item)
        }
        Ok(items)
    }

    pub fn parse_item(&mut self) -> BauResult<ParsedItem> {
        match self.peek_kind() {
            TokenKind::Fn => Ok(ParsedItem::Function(self.parse_function_item()?)),
            TokenKind::Extend => Ok(ParsedItem::Extends(self.parse_extends()?)),
            unknown => Err(self.error(ParserError::UnexpectedToken(unknown, None))),
        }
    }

    pub fn parse_function_item(&mut self) -> BauResult<ParsedFunctionItem> {
        self.consume_specific(TokenKind::Fn)?;
        let ident = self.consume_specific(TokenKind::Identifier)?;
        let name = self.text(ident).to_string();

        self.consume_specific(TokenKind::ParenOpen)?;
        let mut parameters = vec![];
        while !self.at(TokenKind::ParenClose) {
            let param_ident = self.consume_specific(TokenKind::Identifier)?;
            let name = self.text(param_ident).to_string();
            parameters.push(name);
        }
        self.consume_specific(TokenKind::ParenClose)?;

        self.consume_specific(TokenKind::Arrow)?;
        let return_type = self.parse_type()?;

        if !self.at(TokenKind::BraceOpen) {
            let kind = self.peek_kind();
            return Err(self.error(ParserError::UnexpectedToken(
                kind,
                Some(TokenKind::BraceOpen),
            )));
        }
        let body = self.parse_statement()?;

        Ok(ParsedFunctionItem {
            name,
            parameters,
            body,
            return_type,
        })
    }

    pub fn parse_extends(&mut self) -> BauResult<ParsedExtendsItem> {
        self.consume_specific(TokenKind::Extend)?;
        let parsed_type = self.parse_type()?;

        self.consume_specific(TokenKind::BraceOpen)?;
        let mut methods = vec![];
        while !self.at(TokenKind::BraceClose) {
            let method = self.parse_function_item()?;
            methods.push(method);
        }
        self.consume_specific(TokenKind::BraceClose)?;

        Ok(ParsedExtendsItem {
            parsed_type,
            methods,
        })
    }
}
