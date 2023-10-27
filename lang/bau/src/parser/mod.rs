use crate::error::{BauError, BauResult};
use crate::source::Source;
use crate::tokenizer::token::TokenKind;
use crate::tokenizer::{Token, Tokenizer};

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedItem {
    Function(ParsedFunctionItem),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFunctionItem {
    pub name: String,
    pub arguments: Vec<ParsedFunctionArgument>,
    pub return_type_name: String,
    pub body: Vec<ParsedStatement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFunctionArgument {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parser<'source> {
    source: &'source Source<'source>,
    tokens: Vec<Token>,
    cursor: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedStatement {}

impl<'source> Parser<'source> {
    pub fn new(source: &'source Source) -> Self {
        let mut tokens = Tokenizer::new(source.text()).tokenize();
        preprocess_tokens(&mut tokens);
        Self {
            source,
            tokens,
            cursor: 0,
        }
    }

    pub fn parse_top_level(&mut self) -> BauResult<Vec<ParsedItem>> {
        let mut items = vec![];
        while !self.done() {
            match self.parse_item()? {
                Some(item) => items.push(item),
                _ => {
                    return Err(BauError::ExpectedItem {
                        token: self.peek()?.clone(),
                    })
                }
            }
        }
        Ok(items)
    }

    fn parse_item(&mut self) -> BauResult<Option<ParsedItem>> {
        match self.peek_kind()? {
            TokenKind::Fn => self
                .parse_function_item()
                .map(|f| f.map(ParsedItem::Function)),
            _ => Ok(None),
        }
    }

    fn parse_function_item(&mut self) -> BauResult<Option<ParsedFunctionItem>> {
        self.consume_specific(TokenKind::Fn)?;

        let name_ident = self.consume_specific(TokenKind::Identifier)?;
        let name = self.text(&name_ident);

        self.consume_specific(TokenKind::ParenOpen)?;
        let arguments = self.parse_function_arguments()?;
        self.consume_specific(TokenKind::ParenClose)?;

        self.consume_specific(TokenKind::Arrow)?;
        let return_type_ident = self.consume_specific(TokenKind::Identifier)?;
        let return_type_name = self.text(&return_type_ident);

        self.consume_specific(TokenKind::BraceOpen)?;
        let body = self.parse_statement_list()?;
        self.consume_specific(TokenKind::BraceClose)?;

        Ok(Some(ParsedFunctionItem {
            name,
            arguments,
            return_type_name,
            body,
        }))
    }

    fn parse_function_arguments(&mut self) -> BauResult<Vec<ParsedFunctionArgument>> {
        let mut arguments = vec![];
        self.parse_next_function_argument(&mut arguments)?;
        Ok(arguments)
    }

    fn parse_next_function_argument(
        &mut self,
        arguments: &mut Vec<ParsedFunctionArgument>,
    ) -> BauResult<()> {
        if let Some(argument) = self.parse_function_argument()? {
            arguments.push(argument);
            if self.consume_if(TokenKind::Comma) {
                self.parse_next_function_argument(arguments)?;
            }
        }
        Ok(())
    }

    fn parse_function_argument(&mut self) -> BauResult<Option<ParsedFunctionArgument>> {
        let type_ident = self.consume_specific(TokenKind::Identifier)?;
        let type_name = self.text(&type_ident);

        let name_ident = self.consume_specific(TokenKind::Identifier)?;
        let name = self.text(&name_ident);

        Ok(Some(ParsedFunctionArgument { name, type_name }))
    }

    fn parse_statement_list(&mut self) -> BauResult<Vec<ParsedStatement>> {
        let mut statements = vec![];
        while self.peek_kind() != Ok(TokenKind::BraceClose) {
            if let Some(statement) = self.parse_statement()? {
                statements.push(statement);
            } else {
                break;
            }
        }
        Ok(statements)
    }

    fn parse_statement(&mut self) -> BauResult<Option<ParsedStatement>> {
        Ok(None)
    }

    fn peek(&self) -> BauResult<&Token> {
        self.tokens
            .iter()
            .nth(self.cursor)
            .map(Ok)
            .unwrap_or(Err(BauError::UnexpectedEndOfFile {
                token: self
                    .tokens
                    .last()
                    .cloned()
                    .expect("input should have at least one character"),
            }))
    }

    fn peek_kind(&self) -> BauResult<TokenKind> {
        self.peek().map(|t| t.kind)
    }

    fn consume(&mut self) -> BauResult<Token> {
        let token = self.peek()?.clone();
        self.cursor += 1;
        Ok(token)
    }

    fn consume_specific(&mut self, expected: TokenKind) -> BauResult<Token> {
        let token = self.consume()?.clone();
        if !token.is(expected) {
            return Err(BauError::UnexpectedToken { token, expected });
        }
        Ok(token)
    }

    fn consume_if(&mut self, expected: TokenKind) -> bool {
        if self.peek_kind() == Ok(expected) {
            self.consume().unwrap();
            true
        } else {
            false
        }
    }

    fn text(&self, token: &Token) -> String {
        self.source.text()[token.span.start..token.span.end].to_string()
    }

    fn done(&self) -> bool {
        self.cursor >= self.tokens.len()
    }
}

pub fn preprocess_tokens(tokens: &mut Vec<Token>) {
    tokens.retain(|token| {
        !token.is(TokenKind::Whitespace)
            && !token.is(TokenKind::Comment)
            && !token.is(TokenKind::EndOfLine)
    });
}
