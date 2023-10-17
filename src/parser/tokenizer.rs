#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Illegal(char),

    Fn,
    Let,

    ParenOpen,
    ParenClose,
    BraceOpen,
    BraceClose,
    Semicolon,
    Equals,

    Identifier(String),
    NumericLiteral(f64),
}

#[derive(Debug, Clone)]
pub struct Tokenizer<'a> {
    pub tokens: Vec<Token>,
    cursor: usize,
    input: &'a str,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            tokens: vec![],
            cursor: 0,
            input,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = vec![];

        while let Some(token) = self.parse_token() {
            tokens.push(token);
        }

        tokens
    }

    fn parse_token(&mut self) -> Option<Token> {
        self.consume_whitespace();

        if let Some(literal) = self.parse_literal() {
            Some(literal)
        } else if let Some(punctuator) = self.parse_punctuator() {
            Some(punctuator)
        } else if let Some(identifier) = self.parse_identifier() {
            Some(identifier)
        } else if self.peek().is_some() {
            let char = self.consume().unwrap();
            Some(Token::Illegal(char))
        } else {
            None
        }
    }

    fn consume_whitespace(&mut self) {
        while let Some(char) = self.peek() {
            if char.is_ascii_whitespace() {
                self.consume();
            } else {
                break;
            }
        }
    }

    fn parse_literal(&mut self) -> Option<Token> {
        if let Some(numeric) = self.parse_numeric_literal() {
            Some(numeric)
        } else {
            None
        }
    }

    fn parse_numeric_literal(&mut self) -> Option<Token> {
        match self.peek() {
            Some(char) => match char {
                '+' | '-' | '.' | '0'..='9' => {
                    let char = self.consume().unwrap();
                    let mut number_string = String::new();
                    number_string.push(char);

                    let mut is_decimal = char == '.';

                    while let Some(c) = self.peek() {
                        match c {
                            '0'..='9' => {
                                number_string.push(c);
                                self.consume();
                            }
                            '.' => {
                                if is_decimal {
                                    break;
                                } else {
                                    is_decimal = true;
                                    number_string.push(c);
                                    self.consume();
                                }
                            }
                            _ => break,
                        }
                    }

                    Some(Token::NumericLiteral(number_string.parse().unwrap()))
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn parse_punctuator(&mut self) -> Option<Token> {
        match self.consume() {
            Some(char) => match char {
                '(' => Some(Token::ParenOpen),
                ')' => Some(Token::ParenClose),
                '{' => Some(Token::BraceOpen),
                '}' => Some(Token::BraceClose),
                ';' => Some(Token::Semicolon),
                '=' => Some(Token::Equals),
                _ => {
                    self.unconsume();
                    None
                }
            },
            _ => None,
        }
    }

    fn parse_identifier(&mut self) -> Option<Token> {
        match self.peek() {
            Some(char) => match char {
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut identifier_string = String::new();
                    while let Some(c) = self.peek() {
                        match c {
                            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                                identifier_string.push(c);
                                self.consume();
                            }
                            _ => break,
                        }
                    }

                    match identifier_string.as_str() {
                        "fn" => Some(Token::Fn),
                        "let" => Some(Token::Let),
                        _ => Some(Token::Identifier(identifier_string)),
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.cursor)
    }

    fn consume(&mut self) -> Option<char> {
        let c = self.peek();
        self.cursor += 1;
        c
    }

    fn unconsume(&mut self) {
        self.cursor -= 1;
    }
}
