use crate::parser::ast::{Expr, Literal};
use crate::parser::Parser;
use crate::tokenizer::token::{Token, TokenKind};

impl<'input, I> Parser<'input, I>
where
    I: Iterator<Item = Token>,
{
    pub fn parse_expression(&mut self) -> Expr {
        match self.peek() {
            TokenKind::Error => {
                let error_token = self.next().expect("Expected Error");
                let text = self.text(error_token);
                panic!("Error while parsing expression at: {}", text);
            }
            literal @ (TokenKind::IntLiteral
            | TokenKind::FloatLiteral
            | TokenKind::StringLiteral) => {
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
                Expr::Literal(literal)
            }
            TokenKind::Identifier => {
                let name = {
                    let token = self.next().expect("Expected identifier");
                    self.text(token).to_string()
                };

                // Plain identifier
                if !self.at(TokenKind::ParenOpen) {
                    return Expr::Ident(name);
                }

                // Function call
                let mut args = vec![];
                self.consume(TokenKind::ParenOpen);
                while !self.at(TokenKind::ParenClose) {
                    let arg = self.parse_expression();
                    args.push(arg);
                    if self.at(TokenKind::Comma) {
                        self.consume(TokenKind::Comma);
                    }
                }
                self.consume(TokenKind::ParenClose);
                Expr::FnCall { name, args }
            }
            TokenKind::ParenOpen => {
                self.consume(TokenKind::ParenOpen);
                let expr = self.parse_expression();
                self.consume(TokenKind::ParenClose);
                expr
            }
            op @ (TokenKind::Plus | TokenKind::Minus | TokenKind::ExclamationMark) => {
                self.consume(op);
                let expr = self.parse_expression();
                Expr::PrefixOp {
                    op,
                    expr: Box::new(expr),
                }
            }
            kind => panic!("Invalid start of expression: {:?}", kind),
        }
    }
}
