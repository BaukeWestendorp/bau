use crate::error::{BauError, BauResult};
use crate::node::AstNode;
use crate::parser::tokenizer::{Token, TokenKind, Tokenizer};

mod tokenizer;

#[derive(Debug, Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    line: usize,
    column: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Self {
            tokens: Tokenizer::new(input).tokenize(),
            line: 1,
            column: 1,
        }
    }

    pub fn parse(&mut self) -> BauResult<Option<AstNode>> {
        match self.parse_statement_list()? {
            Some(statements) => Ok(Some(AstNode::Program(statements))),
            None => Ok(None),
        }
    }

    fn error(&self, message: String) -> BauError {
        BauError::ParseError {
            line: self.line,
            column: self.column,
            message,
        }
    }

    fn parse_declaration(&mut self) -> BauResult<Option<AstNode>> {
        if let Some(fn_declaration) = self.parse_fn_declaration()? {
            Ok(Some(fn_declaration))
        } else if let Some(let_declaration) = self.parse_let_declaration()? {
            Ok(Some(let_declaration))
        } else {
            Ok(None)
        }
    }

    fn parse_fn_declaration(&mut self) -> BauResult<Option<AstNode>> {
        if self.consume_maybe(&TokenKind::Fn).is_none() {
            return Ok(None);
        }

        match self.peek().cloned() {
            Some(TokenKind::Identifier(name)) => {
                self.consume()?;

                self.consume_certain(&TokenKind::ParenOpen)?;
                // FIXME: Implement parameters parsing
                self.consume_certain(&TokenKind::ParenClose)?;

                let body = self.parse_block()?;

                if body.is_none() {
                    return Err(self.error("Expected block".to_string()));
                }

                Ok(Some(AstNode::FnDeclaration {
                    name: name.clone(),
                    parameters: vec![],
                    body: Box::new(body.unwrap()),
                }))
            }
            _ => return Ok(None),
        }
    }

    fn parse_block(&mut self) -> BauResult<Option<AstNode>> {
        if self.consume_maybe(&TokenKind::BraceOpen).is_none() {
            return Ok(None);
        };
        let body = self.parse_statement_list()?;
        self.consume_maybe(&TokenKind::BraceClose);
        Ok(Some(AstNode::Block(body.unwrap_or(vec![]))))
    }

    fn parse_let_declaration(&mut self) -> BauResult<Option<AstNode>> {
        if self.consume_maybe(&TokenKind::Let).is_none() {
            return Ok(None);
        }

        match self.peek().cloned() {
            Some(TokenKind::Identifier(name)) => {
                self.consume_certain(&TokenKind::Identifier(name.clone()))?;
                self.consume_certain(&TokenKind::Equals)?;

                let value = self.parse_expression()?;

                if value.is_none() {
                    return Err(self.error("Expected expression".to_string()));
                }

                Ok(Some(AstNode::LetDeclaration {
                    name: name.clone(),
                    value: value.map(Box::new),
                }))
            }
            _ => return Err(self.error("Expected identifier".to_string())),
        }
    }

    fn parse_statement_list(&mut self) -> BauResult<Option<Vec<AstNode>>> {
        let mut statements = vec![];
        while let Some(statement) = self.parse_statement()? {
            statements.push(statement);
        }
        Ok(Some(statements))
    }

    fn parse_statement(&mut self) -> BauResult<Option<AstNode>> {
        self.consume_maybe(&TokenKind::Semicolon);

        if let Some(declaration) = self.parse_declaration()? {
            Ok(Some(declaration))
        } else if let Some(expression) = self.parse_expression()? {
            Ok(Some(expression))
        } else {
            Ok(None)
        }
    }

    fn parse_expression(&mut self) -> BauResult<Option<AstNode>> {
        if let Some(block) = self.parse_block()? {
            Ok(Some(block))
        } else if let Some(number_literal) = self.parse_number_literal()? {
            Ok(Some(number_literal))
        } else {
            Ok(None)
        }
    }

    fn parse_number_literal(&mut self) -> BauResult<Option<AstNode>> {
        match self.peek().cloned() {
            Some(TokenKind::NumericLiteral(number)) => {
                self.consume()?;
                Ok(Some(AstNode::NumberLiteral(number)))
            }
            _ => Ok(None),
        }
    }

    fn peek(&self) -> Option<&TokenKind> {
        self.tokens.get(0).map(|t| &t.kind)
    }

    fn consume(&mut self) -> BauResult<TokenKind> {
        let token = match self.tokens.is_empty() {
            true => return Err(self.error("Unexpected EOF".to_string())),
            false => self.tokens.remove(0),
        };

        self.line = token.line;
        self.column = token.column;
        Ok(token.kind)
    }

    fn consume_maybe(&mut self, token_kind: &TokenKind) -> Option<TokenKind> {
        match self.peek() {
            Some(token) if token == token_kind => Some(self.consume().unwrap()),
            _ => None,
        }
    }

    fn consume_certain(&mut self, token_kind: &TokenKind) -> BauResult<TokenKind> {
        match self.consume_maybe(token_kind) {
            Some(token) => Ok(token),
            None => Err(self.error(format!(
                "Expected {:?}, got {}",
                token_kind,
                self.peek()
                    .map_or("EOF Token".to_string(), |t| format!("{:?}", t))
            ))),
        }
    }
}
