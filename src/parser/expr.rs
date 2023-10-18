use crate::builtins;
use crate::error::BauResult;
use crate::parser::ast::{Expr, Literal};
use crate::parser::Parser;
use crate::tokenizer::token::{Token, TokenKind};

impl<'input, I> Parser<'input, I>
where
    I: Iterator<Item = Token>,
{
    pub fn parse_expression(&mut self) -> BauResult<Expr> {
        match self.peek() {
            TokenKind::Error => {
                let error_token = self.next().expect("Expected Error");
                let text = self.text(error_token);
                return Err(self.error(format!("Error while parsing expression at: {}", text)));
            }
            TokenKind::IntLiteral | TokenKind::FloatLiteral | TokenKind::StringLiteral => {
                self.parse_literal_expression()
            }
            TokenKind::Identifier => {
                let name = {
                    let token = self.next().expect("Expected identifier");
                    self.text(token).to_string()
                };

                // Plain identifier
                if !self.at(TokenKind::ParenOpen) {
                    return Ok(Expr::Ident(name));
                }

                // Function call
                let mut args = vec![];
                self.consume(TokenKind::ParenOpen)?;
                while !self.at(TokenKind::ParenClose) {
                    let arg = self.parse_expression()?;
                    args.push(arg);
                    if self.at(TokenKind::Comma) {
                        self.consume(TokenKind::Comma)?;
                    }
                }
                self.consume(TokenKind::ParenClose)?;

                if let Some(function) = builtins::from_name(&name) {
                    return Ok(Expr::BuiltinFnCall { function, args });
                }

                Ok(Expr::FnCall { name, args })
            }
            TokenKind::ParenOpen => {
                self.consume(TokenKind::ParenOpen)?;
                let expr = self.parse_expression();
                self.consume(TokenKind::ParenClose)?;
                expr
            }
            TokenKind::Plus | TokenKind::Minus | TokenKind::ExclamationMark => {
                self.parse_prefix_operator_expression()
            }
            invalid_kind => {
                Err(self.error(format!("Invalid start of expression: `{:?}`", invalid_kind)))
            }
        }
    }

    pub fn parse_literal_expression(&mut self) -> BauResult<Expr> {
        let literal = self.peek();
        let text = {
            let token = self.next().expect("Expected literal");
            self.text(token)
        };
        let literal = match literal {
            TokenKind::IntLiteral => Literal::Int(
                text.parse()
                    .expect(&format!("Invalid integer literal: {}", text)),
            ),
            TokenKind::FloatLiteral => Literal::Float(
                text.parse()
                    .expect(&format!("Invalid float literal: {}", text)),
            ),
            TokenKind::StringLiteral => Literal::String(text.to_string()),
            _ => unreachable!(),
        };
        Ok(Expr::Literal(literal))
    }

    pub fn parse_prefix_operator_expression(&mut self) -> BauResult<Expr> {
        let op = self.next().expect("Expected operator").kind;
        let expr = self.parse_expression()?;
        Ok(Expr::PrefixOp {
            op,
            expr: Box::new(expr),
        })
    }
}
