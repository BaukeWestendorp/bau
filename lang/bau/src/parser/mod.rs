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
    PrefixOperator {
        operator: TokenKind,
        expression: Box<ParsedExpression>,
    },
    InfixOperator {
        operator: TokenKind,
        left: Box<ParsedExpression>,
        right: Box<ParsedExpression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedExpression {
    kind: ParsedExpressionKind,
    range: CodeRange,
}

impl ParsedExpression {
    pub fn new(kind: ParsedExpressionKind, range: CodeRange) -> Self {
        Self { kind, range }
    }

    pub fn kind(&self) -> &ParsedExpressionKind {
        &self.kind
    }

    pub fn range(&self) -> &CodeRange {
        &self.range
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
                        self.peek()?.range(),
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
                        CodeRange::from_ranges(start.range(), end.range()),
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

    fn parse_function_arguments(&mut self) -> ParserResult<Vec<ParsedExpression>> {
        let mut arguments = vec![];
        self.parse_next_function_argument(&mut arguments)?;
        Ok(arguments)
    }

    fn parse_next_function_argument(
        &mut self,
        arguments: &mut Vec<ParsedExpression>,
    ) -> ParserResult<()> {
        if self.peek_kind() == Ok(TokenKind::ParenClose) {
            return Ok(());
        }

        if let Some(argument) = self.parse_expression()? {
            arguments.push(argument);
            if self.consume_if(TokenKind::Comma) {
                self.parse_next_function_argument(arguments)?;
            }
        }
        Ok(())
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
        let start = self.current_token_range()?;
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
                self.peek()?.range(),
            ));
        }

        self.consume_specific(TokenKind::Semicolon)?;

        Ok(Some(ParsedStatement::new(
            ParsedStatementKind::Let {
                name,
                type_name,
                initial_value: initial_value.unwrap(),
            },
            CodeRange::from_ranges(start, self.current_token_range()?),
        )))
    }

    fn parse_return_statement(&mut self) -> ParserResult<Option<ParsedStatement>> {
        let start = self.current_token_range()?;

        self.consume_specific(TokenKind::Return)?;
        let expr = self.parse_expression()?;
        let end = self.current_token_range()?;

        self.consume_specific(TokenKind::Semicolon)?;

        Ok(Some(ParsedStatement::new(
            ParsedStatementKind::Return { value: expr },
            CodeRange::from_ranges(start, end),
        )))
    }

    fn parse_expression(&mut self) -> ParserResult<Option<ParsedExpression>> {
        self.parse_pratt_expression(0)
    }

    fn parse_pratt_expression(
        &mut self,
        min_binding_power: u8,
    ) -> ParserResult<Option<ParsedExpression>> {
        let start = self.current_token_range()?;

        let mut lhs = self.parse_primary_expression(false)?;
        while let op @ (TokenKind::Plus
        | TokenKind::Minus
        | TokenKind::Asterisk
        | TokenKind::Slash
        | TokenKind::Percent
        | TokenKind::EqualsEquals
        | TokenKind::ExclamationMarkEquals
        | TokenKind::LessThan
        | TokenKind::LessThanEquals
        | TokenKind::GreaterThan
        | TokenKind::GreaterThanEquals
        | TokenKind::AmpersandAmpersand
        | TokenKind::PipePipe) = self.peek_kind()?
        {
            if let Some((left_binding_power, right_binding_power)) = infix_binding_power(op) {
                if left_binding_power < min_binding_power {
                    break;
                }

                self.consume_specific(op)?;
                let rhs = self.parse_pratt_expression(right_binding_power)?;
                let end = self.current_token_range()?;
                lhs = Some(ParsedExpression::new(
                    ParsedExpressionKind::InfixOperator {
                        operator: op,
                        left: Box::new(lhs.unwrap()),
                        right: Box::new(rhs.unwrap()),
                    },
                    CodeRange::from_ranges(start, end),
                ));

                continue;
            }
            break;
        }

        Ok(lhs)
    }

    fn parse_primary_expression(
        &mut self,
        ignore_members: bool,
    ) -> ParserResult<Option<ParsedExpression>> {
        match self.peek_kind()? {
            TokenKind::IntLiteral
            | TokenKind::FloatLiteral
            | TokenKind::StringLiteral
            | TokenKind::BoolLiteral => self.parse_literal_expression(),
            TokenKind::Identifier => match self.peek_kind_at(1) {
                Ok(TokenKind::ParenOpen) => self.parse_function_call_expression(),
                Ok(TokenKind::Period) if !ignore_members => match self.peek_kind_at(2) {
                    Ok(TokenKind::Identifier) => match self.peek_kind_at(3)? {
                        TokenKind::ParenOpen => todo!("Implement method calls"),
                        _ => todo!("Implement member access"),
                    },
                    _ => Ok(None),
                },
                Err(_) => Ok(None),
                _ => self.parse_identifier_expression(),
            },
            TokenKind::Plus | TokenKind::Minus | TokenKind::ExclamationMark => {
                self.parse_prefix_operator_expression()
            }
            TokenKind::ParenOpen => {
                self.consume_specific(TokenKind::ParenOpen)?;
                let expr = self.parse_pratt_expression(0);
                self.consume_specific(TokenKind::ParenClose)?;
                expr
            }
            invalid_kind => Err(ParserError::new(
                ParserErrorKind::InvalidExpressionStart {
                    found: invalid_kind,
                },
                self.current_token_range()?,
            )),
        }
    }

    fn parse_prefix_operator_expression(&mut self) -> ParserResult<Option<ParsedExpression>> {
        let token = self.consume()?;
        match token.kind() {
            TokenKind::Plus | TokenKind::Minus | TokenKind::ExclamationMark => {
                let end = self.current_token_range()?;
                if let Some(expression) = self.parse_primary_expression(false)? {
                    Ok(Some(ParsedExpression::new(
                        ParsedExpressionKind::PrefixOperator {
                            operator: token.kind(),
                            expression: Box::new(expression),
                        },
                        CodeRange::from_ranges(token.range(), end),
                    )))
                } else {
                    Err(ParserError::new(
                        ParserErrorKind::ExpectedExpression {
                            found: self.peek_kind()?,
                        },
                        CodeRange::from_ranges(token.range(), end),
                    ))
                }
            }
            _ => Ok(None),
        }
    }

    fn parse_literal_expression(&mut self) -> ParserResult<Option<ParsedExpression>> {
        let token = self.peek()?.clone();
        let literal = match token.kind() {
            TokenKind::IntLiteral => {
                let string_value = self.consume_specific(TokenKind::IntLiteral)?;
                let string_value_text = self.text(&string_value);
                let value = string_value_text.parse::<i64>().unwrap();
                ParsedLiteralExpression::Integer(value)
            }
            TokenKind::FloatLiteral => {
                let string_value = self.consume_specific(TokenKind::FloatLiteral)?;
                let string_value_text = self.text(&string_value);
                let value = string_value_text.parse::<f64>().unwrap();
                ParsedLiteralExpression::Float(value)
            }
            TokenKind::StringLiteral => {
                let string_value = self.consume_specific(TokenKind::StringLiteral)?;
                let string_value_text = self.text(&string_value);
                let value = string_value_text[1..string_value_text.len() - 1].to_string();
                ParsedLiteralExpression::String(value)
            }
            TokenKind::BoolLiteral => {
                let string_value = self.consume_specific(TokenKind::BoolLiteral)?;
                let string_value_text = self.text(&string_value);
                let value = string_value_text.parse::<bool>().unwrap();
                ParsedLiteralExpression::Boolean(value)
            }
            _ => return Ok(None),
        };

        Ok(Some(ParsedExpression::new(
            ParsedExpressionKind::Literal(literal),
            token.range(),
        )))
    }

    fn parse_function_call_expression(&mut self) -> ParserResult<Option<ParsedExpression>> {
        let start = self.current_token_range()?;
        let name = self.parse_identifier()?;
        self.consume_specific(TokenKind::ParenOpen)?;
        let arguments = self.parse_function_arguments()?;
        let end = self.current_token_range()?;
        self.consume_specific(TokenKind::ParenClose)?;
        Ok(Some(ParsedExpression::new(
            ParsedExpressionKind::FunctionCall { name, arguments },
            CodeRange::from_ranges(start, end),
        )))
    }

    fn parse_identifier_expression(&mut self) -> ParserResult<Option<ParsedExpression>> {
        let token = self.peek()?.clone();
        let ident = self.parse_identifier()?;

        match self.peek_kind()? {
            TokenKind::ParenOpen => self.parse_function_call_expression(),
            _ => Ok(Some(ParsedExpression::new(
                ParsedExpressionKind::Variable(ident),
                token.range(),
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

    fn current_token_range(&self) -> ParserResult<CodeRange> {
        self.peek().map(|token| token.range())
    }

    fn peek(&self) -> ParserResult<&Token> {
        self.peek_at(0)
    }

    fn peek_at(&self, offset: usize) -> ParserResult<&Token> {
        self.tokens
            .get(self.cursor + offset)
            .map(Ok)
            .unwrap_or(Err(ParserError::new(
                ParserErrorKind::UnexpectedEndOfFile,
                self.tokens
                    .last()
                    .cloned()
                    .expect("input should have at least one character")
                    .range(),
            )))
    }

    fn peek_kind(&self) -> ParserResult<TokenKind> {
        self.peek_kind_at(0)
    }

    fn peek_kind_at(&self, offset: usize) -> ParserResult<TokenKind> {
        self.peek_at(offset).map(|token| token.kind())
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
                    found: token.kind(),
                    expected,
                },
                token.range(),
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
        self.source.text()[token.range().span.start..token.range().span.end].to_string()
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

fn infix_binding_power(op: TokenKind) -> Option<(u8, u8)> {
    match op {
        TokenKind::PipePipe => Some((1, 2)),
        TokenKind::AmpersandAmpersand => Some((3, 4)),
        TokenKind::EqualsEquals | TokenKind::ExclamationMarkEquals => Some((5, 6)),
        TokenKind::LessThan
        | TokenKind::LessThanEquals
        | TokenKind::GreaterThan
        | TokenKind::GreaterThanEquals => Some((7, 8)),
        TokenKind::Plus | TokenKind::Minus => Some((9, 10)),
        TokenKind::Asterisk | TokenKind::Slash | TokenKind::Percent => Some((11, 12)),
        _ => None,
    }
}
