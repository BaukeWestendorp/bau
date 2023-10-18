use crate::tokenizer::token::TokenKind;
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Rule {
    pub kind: TokenKind,
    pub matches: fn(&str) -> Option<usize>,
}

fn match_single_char(input: &str, char: char) -> Option<usize> {
    input
        .chars()
        .next()
        .and_then(|ch| if char == ch { Some(1) } else { None })
}

fn match_two_chars(input: &str, first: char, second: char) -> Option<usize> {
    if input.len() >= 2 {
        match_single_char(input, first)
            .and_then(|_| match_single_char(&input[1..], second).map(|_| 2))
    } else {
        None
    }
}

fn match_keyword(input: &str, keyword: &str) -> Option<usize> {
    input.starts_with(keyword).then(|| keyword.len())
}

fn match_regex(input: &str, r: &Regex) -> Option<usize> {
    r.find(input).map(|regex_match| regex_match.end())
}

lazy_static! {
    static ref STRING_REGEX: Regex = Regex::new(r#"^"((\\"|\\\\)|[^\\"])*""#).unwrap();
    static ref FLOAT_REGEX: Regex =
        Regex::new(r#"^((\d+(\.\d+)?)|(\.\d+))([Ee](\+|-)?\d+)?"#).unwrap();
    static ref IDENTIFIER_REGEX: Regex = Regex::new(r##"^([A-Za-z]|_)([A-Za-z]|_|\d)*"##).unwrap();
}

pub(crate) fn get_rules() -> Vec<Rule> {
    macro_rules! char {
        ($token:ident, $char:literal) => {
            Rule {
                kind: TokenKind::$token,
                matches: |input| match_single_char(input, $char),
            }
        };
    }

    macro_rules! keyword {
        ($token:ident, $keyword:literal) => {
            Rule {
                kind: TokenKind::$token,
                matches: |input| match_keyword(input, $keyword),
            }
        };
    }

    macro_rules! regex {
        ($token:ident, $regex:expr) => {
            Rule {
                kind: TokenKind::$token,
                matches: |input| match_regex(input, $regex),
            }
        };
    }

    vec![
        char!(ParenOpen, '('),
        char!(ParenClose, ')'),
        char!(BraceOpen, '{'),
        char!(BraceClose, '}'),
        char!(SquareOpen, '['),
        char!(SquareClose, ']'),
        char!(Semicolon, ';'),
        char!(Comma, ','),
        char!(Equals, '='),
        char!(Plus, '+'),
        char!(Minus, '-'),
        char!(ExclamationMark, '!'),
        keyword!(Let, "let"),
        keyword!(Fn, "fn"),
        keyword!(If, "if"),
        keyword!(Return, "return"),
        regex!(StringLiteral, &STRING_REGEX),
        regex!(FloatLiteral, &FLOAT_REGEX),
        regex!(Identifier, &IDENTIFIER_REGEX),
    ]
}
