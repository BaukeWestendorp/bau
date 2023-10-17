#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
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

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, column: usize) -> Self {
        Self { kind, line, column }
    }
}

#[derive(Debug, Clone)]
pub struct Tokenizer<'a> {
    pub tokens: Vec<Token>,
    cursor: usize,
    input: &'a str,
    line: usize,
    column: usize,
    p_column: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            tokens: vec![],
            cursor: 0,
            input,
            line: 1,
            column: 1,
            p_column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = vec![];

        while let Some(token) = self.parse_token() {
            tokens.push(token);
        }

        tokens
    }

    fn token(&self, kind: TokenKind) -> Token {
        Token::new(kind, self.line, self.column)
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
            Some(self.token(TokenKind::Illegal(char)))
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

                    Some(self.token(TokenKind::NumericLiteral(number_string.parse().unwrap())))
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn parse_punctuator(&mut self) -> Option<Token> {
        match self.consume() {
            Some(char) => match char {
                '(' => Some(self.token(TokenKind::ParenOpen)),
                ')' => Some(self.token(TokenKind::ParenClose)),
                '{' => Some(self.token(TokenKind::BraceOpen)),
                '}' => Some(self.token(TokenKind::BraceClose)),
                ';' => Some(self.token(TokenKind::Semicolon)),
                '=' => Some(self.token(TokenKind::Equals)),
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
                        "fn" => Some(self.token(TokenKind::Fn)),
                        "let" => Some(self.token(TokenKind::Let)),
                        _ => Some(self.token(TokenKind::Identifier(identifier_string))),
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
        self.column += 1;
        if c == Some('\n') {
            self.line += 1;
            self.column = 1;
        }
        self.p_column = self.column;
        c
    }

    fn unconsume(&mut self) {
        self.cursor -= 1;
        self.column -= 1;
        if self.input.chars().nth(self.cursor) == Some('\n') {
            self.line -= 1;
            self.column = self.p_column;
        }
        self.p_column = self.column;
    }
}
