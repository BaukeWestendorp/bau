use crate::builtins::BuiltinFunction;
use crate::error::{BauError, BauResult, ParserError};
use crate::parser::ast::{BlockKind, Literal};
use crate::parser::source::Source;
use crate::tokenizer::token::{Span, Token, TokenKind};
use crate::tokenizer::Tokenizer;

pub mod ast;
pub mod expr;
pub mod item;
pub mod source;
pub mod stmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedType {
    Void,
    Int,
    Float,
    String,
    Bool,
    Name(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedStmt {
    Let {
        name: String,
        parsed_type: ParsedType,
        expr: ParsedExpr,
    },
    Assignment {
        name: String,
        expr: ParsedExpr,
    },
    If {
        condition: ParsedExpr,
        then_branch: Box<ParsedStmt>,
        else_branch: Option<Box<ParsedStmt>>,
    },
    Loop {
        body: Box<ParsedStmt>,
    },
    Block {
        block_kind: BlockKind,
        statements: Vec<ParsedStmt>,
    },
    Return {
        expr: Option<ParsedExpr>,
    },
    Continue,
    Break,
    Expression {
        expr: ParsedExpr,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedExprKind {
    Literal(Literal),
    Identifier(String),
    BuiltinFnCall {
        function: BuiltinFunction,
        args: Vec<ParsedExpr>,
    },
    FnCall(ParsedFunctionCall),
    PrefixOp {
        op: TokenKind,
        expr: Box<ParsedExpr>,
    },
    InfixOp {
        op: TokenKind,
        lhs: Box<ParsedExpr>,
        rhs: Box<ParsedExpr>,
    },
    PostfixOp {
        op: TokenKind,
        expr: Box<ParsedExpr>,
    },
    MethodCall {
        expr: Box<ParsedExpr>,
        call: ParsedFunctionCall,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedExpr {
    pub kind: ParsedExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFunctionCall {
    pub name: String,
    pub args: Vec<ParsedExpr>,
}

impl std::fmt::Display for ParsedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsedType::Void => write!(f, "void"),
            ParsedType::String => write!(f, "string"),
            ParsedType::Int => write!(f, "int"),
            ParsedType::Float => write!(f, "float"),
            ParsedType::Bool => write!(f, "bool"),
            Self::Name(name) => write!(f, "{}", name),
        }
    }
}

pub struct Parser<'source> {
    source: &'source Source,
    tokens: Vec<Token>,
    cursor: usize,
}

impl<'source> Parser<'source> {
    pub fn new(source: &'source Source) -> Self {
        let mut tokenizer = Tokenizer::new(source.text());
        let mut tokens = vec![];
        while let Some(token) = tokenizer.next() {
            match &token.kind {
                TokenKind::Whitespace => continue,
                TokenKind::Comment => loop {
                    if tokenizer
                        .next()
                        .is_some_and(|token| token.text(source.text()).contains('\n'))
                    {
                        break;
                    }
                },
                _ => tokens.push(token),
            }
        }
        Self {
            source,
            tokens,
            cursor: 0,
        }
    }
}

impl<'source> Parser<'source> {
    /// Get the source text of a token.
    pub fn text(&self, token: Token) -> &'source str {
        token.text(&self.source.text())
    }

    /// Look at the next token's kind without consuming it.
    pub(crate) fn peek_kind(&mut self) -> TokenKind {
        self.peek().kind
    }

    /// Look at the token's kind after the next token without consuming it.
    pub(crate) fn peek_offset_kind(&mut self, offset: isize) -> TokenKind {
        self.peek_offset(offset).kind
    }

    /// Look at the next token without consuming it.
    pub(crate) fn peek(&mut self) -> Token {
        self.tokens
            .get(self.cursor)
            .unwrap_or(&self.source.eof_token())
            .clone()
    }

    /// Look at the token after the next token without consuming it.
    pub(crate) fn peek_offset(&mut self, offset: isize) -> Token {
        let offset = if self.cursor.saturating_add_signed(offset) > self.tokens.len() {
            self.tokens.len() - 1
        } else {
            self.cursor + offset as usize
        };
        self.tokens
            .get(offset)
            .unwrap_or(&self.source.eof_token())
            .clone()
    }

    /// Check if the next token is of a certain kind.
    pub(crate) fn at(&mut self, kind: TokenKind) -> bool {
        self.peek_kind() == kind
    }

    /// Consume the current token and advance the iterator.
    pub(crate) fn consume(&mut self) -> BauResult<Token> {
        let token = self.peek();
        self.cursor += 1;
        match token.kind {
            TokenKind::EndOfFile => Err(self.error(ParserError::UnexpectedEof(None))),
            _ => Ok(token),
        }
    }

    /// Progress the iterator by one token and check if it is of a certain kind.
    pub(crate) fn consume_specific(&mut self, expected: TokenKind) -> BauResult<Token> {
        let current = self.peek();

        if current.kind == TokenKind::Error {
            return Err(self.error(ParserError::UnknownToken(current.kind)));
        }

        if current.kind != expected {
            return Err(self.error(ParserError::UnexpectedToken(current.kind, Some(expected))));
        }

        match self.consume() {
            Ok(token) => Ok(token),
            Err(_) => Err(self.error(ParserError::UnexpectedEof(Some(expected)))),
        }
    }

    pub(crate) fn current_char_cursor(&mut self) -> usize {
        self.peek().span.start
    }

    fn error(&mut self, parser_error: ParserError) -> BauError {
        BauError::ParserError {
            span: self.peek().span,
            error: parser_error,
        }
    }
}
