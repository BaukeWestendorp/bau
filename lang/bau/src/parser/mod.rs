use crate::source::{CodeRange, Source};
use crate::tokenizer::token::TokenKind;
use crate::tokenizer::{Token, Tokenizer};

pub mod error;

pub use error::ParserError;

use self::error::{ParserErrorKind, ParserResult};

#[derive(Debug, Clone, PartialEq)]
pub struct TypeName {
    name: String,
    token: Token,
}

impl TypeName {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn token(&self) -> &Token {
        &self.token
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedItemKind {
    Function(ParsedFunctionItem),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedItem {
    kind: ParsedItemKind,
    range: CodeRange,
}

impl ParsedItem {
    pub fn new(kind: ParsedItemKind, range: CodeRange) -> Self {
        Self { kind, range }
    }

    pub fn kind(&self) -> &ParsedItemKind {
        &self.kind
    }

    pub fn range(&self) -> &CodeRange {
        &self.range
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFunctionItem {
    pub name: String,
    pub parameters: Vec<ParsedFunctionParameter>,
    pub return_type_name: TypeName,
    pub body: Vec<ParsedStatement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFunctionParameter {
    pub name: String,
    pub type_name: TypeName,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parser<'source> {
    source: &'source Source<'source>,
    tokens: Vec<Token>,
    cursor: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedStatementKind {
    Let {
        name: Identifier,
        type_name: TypeName,
        initial_value: ParsedExpression,
    },
    Return {
        value: Option<ParsedExpression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedStatement {
    kind: ParsedStatementKind,
    range: CodeRange,
}

impl ParsedStatement {
    pub fn new(kind: ParsedStatementKind, range: CodeRange) -> Self {
        Self { kind, range }
    }

    pub fn kind(&self) -> &ParsedStatementKind {
        &self.kind
    }

    pub fn range(&self) -> &CodeRange {
        &self.range
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedExpressionKind {
    Literal(ParsedLiteralExpression),
    Variable(Identifier),
    FunctionCall {
        name: Identifier,
        arguments: Vec<ParsedExpression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedExpression {
    kind: ParsedExpressionKind,
    token: Token,
}

impl ParsedExpression {
    pub fn new(kind: ParsedExpressionKind, token: Token) -> Self {
        Self { kind, token }
    }

    pub fn kind(&self) -> &ParsedExpressionKind {
        &self.kind
    }

    pub fn token(&self) -> &Token {
        &self.token
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier {
    name: String,
    token: Token,
}

impl Identifier {
    pub fn new(name: String, token: Token) -> Self {
        Self { name, token }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn token(&self) -> &Token {
        &self.token
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedLiteralExpression {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

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

    pub fn parse_top_level(&mut self) -> ParserResult<Vec<ParsedItem>> {
        let mut items = vec![];
        while !self.done() {
            match self.parse_item()? {
                Some(item) => items.push(item),
                _ => {
                    return Err(ParserError::new(
                        ParserErrorKind::ExpectedItem {
                            found: self.peek_kind()?,
                        },
                        self.peek()?.range.clone(),
                    ))
                }
            }
        }
        Ok(items)
    }

    fn parse_item(&mut self) -> ParserResult<Option<ParsedItem>> {
        let start = self.peek()?.clone();
        match self.peek_kind()? {
            TokenKind::Fn => self.parse_function_item().map(|f| {
                f.map(|f| {
                    let end = self.peek().unwrap().clone();
                    ParsedItem::new(
                        ParsedItemKind::Function(f),
                        CodeRange::from_ranges(start.range, end.range),
                    )
                })
            }),
            _ => Ok(None),
        }
    }

    fn parse_function_item(&mut self) -> ParserResult<Option<ParsedFunctionItem>> {
        self.consume_specific(TokenKind::Fn)?;

        let name_ident = self.consume_specific(TokenKind::Identifier)?;
        let name = self.text(&name_ident);

        self.consume_specific(TokenKind::ParenOpen)?;
        let parameters = self.parse_function_parameters()?;
        self.consume_specific(TokenKind::ParenClose)?;

        self.consume_specific(TokenKind::Arrow)?;

        let return_type_name = self.parse_type_name()?;

        self.consume_specific(TokenKind::BraceOpen)?;
        let body = self.parse_statement_list()?;
        self.consume_specific(TokenKind::BraceClose)?;

        Ok(Some(ParsedFunctionItem {
            name,
            parameters,
            return_type_name,
            body,
        }))
    }

    fn parse_function_parameters(&mut self) -> ParserResult<Vec<ParsedFunctionParameter>> {
        let mut parameters = vec![];
        self.parse_next_function_parameter(&mut parameters)?;
        Ok(parameters)
    }

    fn parse_next_function_parameter(
        &mut self,
        parameters: &mut Vec<ParsedFunctionParameter>,
    ) -> ParserResult<()> {
        if self.peek_kind() == Ok(TokenKind::ParenClose) {
            return Ok(());
        }

        if let Some(parameter) = self.parse_function_parameter()? {
            parameters.push(parameter);
            if self.consume_if(TokenKind::Comma) {
                self.parse_next_function_parameter(parameters)?;
            }
        }
        Ok(())
    }

    fn parse_function_parameter(&mut self) -> ParserResult<Option<ParsedFunctionParameter>> {
        let type_name = self.parse_type_name()?;

        let name_ident = self.consume_specific(TokenKind::Identifier)?;
        let name = self.text(&name_ident);

        Ok(Some(ParsedFunctionParameter { name, type_name }))
    }

    fn parse_statement_list(&mut self) -> ParserResult<Vec<ParsedStatement>> {
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

    fn parse_statement(&mut self) -> ParserResult<Option<ParsedStatement>> {
        match self.peek_kind()? {
            TokenKind::Let => self.parse_let_statement(),
            TokenKind::Return => self.parse_return_statement(),
            _ => Ok(None),
        }
    }

    fn parse_let_statement(&mut self) -> ParserResult<Option<ParsedStatement>> {
        let start = self.peek()?.clone();
        self.consume_specific(TokenKind::Let)?;

        let type_name = self.parse_type_name()?;

        let name = self.parse_identifier()?;

        self.consume_specific(TokenKind::Equals)?;

        let initial_value = self.parse_expression()?;

        if initial_value.is_none() {
            return Err(ParserError::new(
                ParserErrorKind::ExpectedExpression {
                    found: self.peek_kind()?,
                },
                self.peek()?.range.clone(),
            ));
        }

        self.consume_specific(TokenKind::Semicolon)?;

        let end = self.peek()?.clone();

        let range = CodeRange::from_ranges(start.range, end.range);
        Ok(Some(ParsedStatement::new(
            ParsedStatementKind::Let {
                name,
                type_name,
                initial_value: initial_value.unwrap(),
            },
            range,
        )))
    }

    fn parse_return_statement(&mut self) -> ParserResult<Option<ParsedStatement>> {
        let start = self.peek()?.clone();

        self.consume_specific(TokenKind::Return)?;
        let expr = self.parse_expression()?;
        let end = self.peek()?.clone();

        self.consume_specific(TokenKind::Semicolon)?;

        let range = CodeRange::from_ranges(start.range, end.range);
        Ok(Some(ParsedStatement::new(
            ParsedStatementKind::Return { value: expr },
            range,
        )))
    }

    fn parse_expression(&mut self) -> ParserResult<Option<ParsedExpression>> {
        let token = self.peek()?.clone();
        match &token.kind {
            TokenKind::IntLiteral
            | TokenKind::FloatLiteral
            | TokenKind::StringLiteral
            | TokenKind::BoolLiteral => self.parse_literal_expression().map(|expression| {
                expression.map(|expression| {
                    ParsedExpression::new(ParsedExpressionKind::Literal(expression), token)
                })
            }),
            TokenKind::Identifier => self.parse_identifier_expression(),
            _ => Ok(None),
        }
    }

    fn parse_literal_expression(&mut self) -> ParserResult<Option<ParsedLiteralExpression>> {
        match self.peek_kind()? {
            TokenKind::IntLiteral => {
                let string_value = self.consume_specific(TokenKind::IntLiteral)?;
                let string_value_text = self.text(&string_value);
                let value = string_value_text.parse::<i64>().unwrap();
                Ok(Some(ParsedLiteralExpression::Integer(value)))
            }
            TokenKind::FloatLiteral => {
                let string_value = self.consume_specific(TokenKind::FloatLiteral)?;
                let string_value_text = self.text(&string_value);
                let value = string_value_text.parse::<f64>().unwrap();
                Ok(Some(ParsedLiteralExpression::Float(value)))
            }
            TokenKind::StringLiteral => {
                let string_value = self.consume_specific(TokenKind::StringLiteral)?;
                let string_value_text = self.text(&string_value);
                let value = string_value_text[1..string_value_text.len() - 1].to_string();
                Ok(Some(ParsedLiteralExpression::String(value)))
            }
            TokenKind::BoolLiteral => {
                let string_value = self.consume_specific(TokenKind::BoolLiteral)?;
                let string_value_text = self.text(&string_value);
                let value = string_value_text.parse::<bool>().unwrap();
                Ok(Some(ParsedLiteralExpression::Boolean(value)))
            }
            _ => Ok(None),
        }
    }

    fn parse_identifier_expression(&mut self) -> ParserResult<Option<ParsedExpression>> {
        let token = self.peek()?.clone();
        let ident = self.parse_identifier()?;

        match self.peek_kind()? {
            TokenKind::ParenOpen => {
                self.consume_specific(TokenKind::ParenOpen)?;
                // FIXME: Parse arguments
                self.consume_specific(TokenKind::ParenClose)?;
                Ok(Some(ParsedExpression::new(
                    ParsedExpressionKind::FunctionCall {
                        name: ident,
                        arguments: vec![],
                    },
                    token,
                )))
            }
            _ => Ok(Some(ParsedExpression::new(
                ParsedExpressionKind::Variable(ident),
                token,
            ))),
        }
    }

    fn parse_identifier(&mut self) -> ParserResult<Identifier> {
        let ident = self.consume_specific(TokenKind::Identifier)?;
        let name = self.text(&ident);
        Ok(Identifier { name, token: ident })
    }

    fn parse_type_name(&mut self) -> ParserResult<TypeName> {
        let type_ident = self.consume_specific(TokenKind::Identifier)?;
        let name = self.text(&type_ident);
        Ok(TypeName {
            name,
            token: type_ident,
        })
    }

    fn peek(&self) -> ParserResult<&Token> {
        self.peek_at(0)
    }

    fn peek_at(&self, offset: usize) -> ParserResult<&Token> {
        self.tokens
            .iter()
            .nth(self.cursor + offset)
            .map(Ok)
            .unwrap_or(Err(ParserError::new(
                ParserErrorKind::UnexpectedEndOfFile,
                self.tokens
                    .last()
                    .cloned()
                    .expect("input should have at least one character")
                    .range,
            )))
    }

    fn peek_kind(&self) -> ParserResult<TokenKind> {
        self.peek_kind_at(0)
    }

    fn peek_kind_at(&self, offset: usize) -> ParserResult<TokenKind> {
        self.peek_at(offset).map(|token| token.kind)
    }

    fn consume(&mut self) -> ParserResult<Token> {
        let token = self.peek()?.clone();
        self.cursor += 1;
        Ok(token)
    }

    fn consume_specific(&mut self, expected: TokenKind) -> ParserResult<Token> {
        let token = self.consume()?.clone();
        if !token.is(expected) {
            return Err(ParserError::new(
                ParserErrorKind::UnexpectedToken {
                    found: token.kind,
                    expected,
                },
                token.range,
            ));
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
        self.source.text()[token.range.span.start..token.range.span.end].to_string()
    }

    fn done(&self) -> bool {
        self.peek_kind() == Ok(TokenKind::EndOfFile)
    }
}

pub fn preprocess_tokens(tokens: &mut Vec<Token>) {
    tokens.retain(|token| {
        !token.is(TokenKind::Whitespace)
            && !token.is(TokenKind::Comment)
            && !token.is(TokenKind::EndOfLine)
    });
}
