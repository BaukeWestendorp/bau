use crate::builtins;
use crate::error::{BauResult, ParserError};
use crate::parser::ast::Literal;
use crate::parser::{ParsedExpr, ParsedExprKind, ParsedFunctionCall, Parser};
use crate::tokenizer::token::{Span, TokenKind};

trait Operator {
    fn prefix_binding_power(&self) -> ((), u8);
    fn infix_binding_power(&self) -> Option<(u8, u8)>;
    fn postfix_binding_power(&self) -> Option<(u8, ())>;
}

impl Operator for TokenKind {
    fn prefix_binding_power(&self) -> ((), u8) {
        match self {
            TokenKind::Plus | TokenKind::Minus | TokenKind::ExclamationMark => ((), 50),
            _ => unreachable!("Not a prefix operator: `{}`", self),
        }
    }

    fn infix_binding_power(&self) -> Option<(u8, u8)> {
        match self {
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

    fn postfix_binding_power(&self) -> Option<(u8, ())> {
        None
    }
}

impl Parser<'_> {
    fn create_expr(&mut self, cursor_start: usize, kind: ParsedExprKind) -> ParsedExpr {
        ParsedExpr {
            kind,
            span: Span {
                start: cursor_start,
                end: self.current_char_cursor(),
            },
        }
    }

    pub fn parse_expression(&mut self) -> BauResult<ParsedExpr> {
        self.parse_pratt_expression(0)
    }

    pub fn parse_pratt_expression(&mut self, min_binding_power: u8) -> BauResult<ParsedExpr> {
        let cursor_start = self.current_char_cursor();

        let mut lhs = self.parse_primary_expression(false)?;

        loop {
            let op = match self.peek_kind() {
                op @ (TokenKind::Plus
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
                | TokenKind::PipePipe) => op,
                _ => break,
            };

            if let Some((left_binding_power, right_binding_power)) = op.infix_binding_power() {
                if left_binding_power < min_binding_power {
                    break;
                }

                self.consume_specific(op)?;
                let rhs = self.parse_pratt_expression(right_binding_power)?;
                lhs = self.create_expr(
                    cursor_start,
                    ParsedExprKind::InfixOp {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                );

                continue;
            }
            break;
        }

        Ok(lhs)
    }

    pub fn parse_primary_expression(&mut self, ignore_members: bool) -> BauResult<ParsedExpr> {
        match self.peek_kind() {
            TokenKind::IntLiteral
            | TokenKind::FloatLiteral
            | TokenKind::StringLiteral
            | TokenKind::BoolLiteral => self.parse_literal_expression(),
            TokenKind::Identifier => match self.peek_offset_kind(1) {
                TokenKind::ParenOpen => self.parse_function_call_expression(),
                TokenKind::Period if !ignore_members => match self.peek_offset_kind(2) {
                    TokenKind::Identifier => match self.peek_offset_kind(3) {
                        TokenKind::ParenOpen => self.parse_method_call_expression(),
                        _ => todo!(),
                    },
                    _ => todo!(),
                },
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
            invalid_kind => Err(self.error(ParserError::InvalidStartOfExpression(invalid_kind))),
        }
    }

    pub fn parse_function_call_expression(&mut self) -> BauResult<ParsedExpr> {
        let cursor_start = self.current_char_cursor();

        let name = {
            let token = self.consume().expect("Expected identifier");
            self.text(token).to_string()
        };

        let mut args = vec![];
        self.consume_specific(TokenKind::ParenOpen)?;
        while !self.at(TokenKind::ParenClose) {
            let arg = self.parse_pratt_expression(0)?;
            args.push(arg);
            if self.at(TokenKind::Comma) {
                self.consume_specific(TokenKind::Comma)?;
            }
        }
        self.consume_specific(TokenKind::ParenClose)?;

        if let Some(function) = builtins::from_name(&name) {
            return Ok(self.create_expr(
                cursor_start,
                ParsedExprKind::BuiltinFnCall { function, args },
            ));
        }

        Ok(self.create_expr(
            cursor_start,
            ParsedExprKind::FnCall(ParsedFunctionCall { name, args }),
        ))
    }

    pub fn parse_method_call_expression(&mut self) -> BauResult<ParsedExpr> {
        let cursor_start = self.current_char_cursor();

        let expr = self.parse_primary_expression(true)?;

        self.consume_specific(TokenKind::Period)?;

        let name = {
            let token = self.consume().expect("Expected identifier");
            self.text(token).to_string()
        };

        let mut args = vec![];
        self.consume_specific(TokenKind::ParenOpen)?;
        while !self.at(TokenKind::ParenClose) {
            let arg = self.parse_pratt_expression(0)?;
            args.push(arg);
            if self.at(TokenKind::Comma) {
                self.consume_specific(TokenKind::Comma)?;
            }
        }
        self.consume_specific(TokenKind::ParenClose)?;

        Ok(self.create_expr(
            cursor_start,
            ParsedExprKind::MethodCall {
                expr: Box::new(expr),
                call: ParsedFunctionCall { name, args },
            },
        ))
    }

    pub fn parse_identifier_expression(&mut self) -> BauResult<ParsedExpr> {
        let cursor_start = self.current_char_cursor();

        let name = {
            let token = self.consume().expect("Expected identifier");
            self.text(token).to_string()
        };

        Ok(self.create_expr(cursor_start, ParsedExprKind::Identifier(name)))
    }

    pub fn parse_literal_expression(&mut self) -> BauResult<ParsedExpr> {
        let cursor_start = self.current_char_cursor();
        let literal = self.peek_kind();
        let text = {
            let token = self.consume().expect("Expected literal");
            self.text(token)
        };
        let literal = match literal {
            TokenKind::IntLiteral => Literal::Int(
                text.parse()
                    .expect(&format!("Invalid integer literal: `{}`", text)),
            ),
            TokenKind::FloatLiteral => Literal::Float(
                text.parse()
                    .expect(&format!("Invalid float literal: `{}`", text)),
            ),
            TokenKind::StringLiteral => {
                // Remove quotes
                let text = text[1..text.len() - 1].to_string();
                Literal::String(text)
            }
            TokenKind::BoolLiteral => Literal::Bool(
                text.parse()
                    .expect(&format!("Invalid bool literal: `{}`", text)),
            ),
            _ => unreachable!(),
        };
        Ok(self.create_expr(cursor_start, ParsedExprKind::Literal(literal)))
    }

    pub fn parse_prefix_operator_expression(&mut self) -> BauResult<ParsedExpr> {
        let cursor_start = self.current_char_cursor();
        let op = self.consume().expect("Expected operator").kind;
        let expr = self.parse_pratt_expression(0)?;
        Ok(self.create_expr(
            cursor_start,
            ParsedExprKind::PrefixOp {
                op,
                expr: Box::new(expr),
            },
        ))
    }
}