use crate::builtins;
use crate::error::BauResult;
use crate::parser::ast::{Expr, Literal};
use crate::parser::Parser;
use crate::tokenizer::token::TokenKind;

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
            TokenKind::Asterisk | TokenKind::Slash => Some((11, 12)),
            _ => None,
        }
    }

    fn postfix_binding_power(&self) -> Option<(u8, ())> {
        None
    }
}

impl Parser<'_> {
    pub fn parse_expression(&mut self) -> BauResult<Expr> {
        self.parse_pratt_expression(0)
    }

    pub fn parse_pratt_expression(&mut self, min_binding_power: u8) -> BauResult<Expr> {
        let mut lhs = self.parse_primary_expression()?;

        loop {
            let op = match self.peek_kind() {
                op @ (TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Asterisk
                | TokenKind::Slash
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
                lhs = Expr::InfixOp {
                    left: Box::new(lhs),
                    op,
                    right: Box::new(rhs),
                };

                continue;
            }
            break;
        }

        Ok(lhs)
    }

    pub fn parse_primary_expression(&mut self) -> BauResult<Expr> {
        match self.peek_kind() {
            TokenKind::IntLiteral | TokenKind::FloatLiteral | TokenKind::StringLiteral => {
                self.parse_literal_expression()
            }
            TokenKind::Identifier => {
                let name = {
                    let token = self.consume().expect("Expected identifier");
                    self.text(token).to_string()
                };

                // Plain identifier
                if !self.at(TokenKind::ParenOpen) {
                    return Ok(Expr::Identifier(name));
                }

                // Function call
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
                    return Ok(Expr::BuiltinFnCall { function, args });
                }

                Ok(Expr::FnCall { name, args })
            }
            TokenKind::ParenOpen => {
                self.consume_specific(TokenKind::ParenOpen)?;
                let expr = self.parse_pratt_expression(0);
                self.consume_specific(TokenKind::ParenClose)?;
                expr
            }
            TokenKind::Plus | TokenKind::Minus | TokenKind::ExclamationMark => {
                self.parse_prefix_operator_expression()
            }
            invalid_kind => {
                Err(self.error(format!("Invalid start of expression: `{}`", invalid_kind)))
            }
        }
    }

    pub fn parse_literal_expression(&mut self) -> BauResult<Expr> {
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
            TokenKind::StringLiteral => Literal::String(text.to_string()),
            _ => unreachable!(),
        };
        Ok(Expr::Literal(literal))
    }

    pub fn parse_prefix_operator_expression(&mut self) -> BauResult<Expr> {
        let op = self.consume().expect("Expected operator").kind;
        let expr = self.parse_pratt_expression(0)?;
        Ok(Expr::PrefixOp {
            op,
            expr: Box::new(expr),
        })
    }
}
