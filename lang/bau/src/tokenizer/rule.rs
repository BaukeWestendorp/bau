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
    static ref INT_REGEX: Regex = Regex::new(r#"^[+-]?\d+"#).unwrap();
    static ref FLOAT_REGEX: Regex =
        Regex::new(r#"^((\d+(\.\d+)?)|(\.\d+))([Ee](\+|-)?\d+)?"#).unwrap();
    static ref BOOL_REGEX: Regex = Regex::new(r#"^\b(?:true|false)\b"#).unwrap();
    static ref IDENTIFIER_REGEX: Regex = Regex::new(r##"^([A-Za-z]|_)([A-Za-z]|_|\d)*"##).unwrap();
}

pub(crate) fn get_rules() -> Vec<Rule> {
    macro_rules! char {
        ($token:expr) => {
            Rule {
                kind: $token,
                matches: |input| {
                    match_single_char(input, $token.to_string().chars().nth(0).unwrap())
                },
            }
        };
    }

    macro_rules! two_chars {
        ($token:expr, $repr:expr) => {
            Rule {
                kind: $token,
                matches: |input| {
                    match_two_chars(
                        input,
                        $repr.chars().nth(0).unwrap(),
                        $repr.chars().nth(1).unwrap(),
                    )
                },
            }
        };
        ($token:expr) => {
            two_chars!($token, $token.to_string())
        };
    }

    macro_rules! keyword {
        ($token:expr) => {
            Rule {
                kind: $token,
                matches: |input| match_keyword(input, $token.to_string().as_str()),
            }
        };
    }

    macro_rules! regex {
        ($token:expr, $regex:expr) => {
            Rule {
                kind: $token,
                matches: |input| match_regex(input, $regex),
            }
        };
    }

    vec![
        char!(TokenKind::Minus),
        char!(TokenKind::ExclamationMark),
        char!(TokenKind::Equals),
        char!(TokenKind::LessThan),
        char!(TokenKind::GreaterThan),
        char!(TokenKind::Percent),
        two_chars!(TokenKind::Comment, "//"),
        two_chars!(TokenKind::EqualsEquals),
        two_chars!(TokenKind::ExclamationMarkEquals),
        two_chars!(TokenKind::AmpersandAmpersand),
        two_chars!(TokenKind::PipePipe),
        two_chars!(TokenKind::LessThanEquals),
        two_chars!(TokenKind::GreaterThanEquals),
        two_chars!(TokenKind::Arrow),
        keyword!(TokenKind::Extend),
        keyword!(TokenKind::Fn),
        keyword!(TokenKind::Let),
        keyword!(TokenKind::If),
        keyword!(TokenKind::Else),
        keyword!(TokenKind::Loop),
        keyword!(TokenKind::Return),
        keyword!(TokenKind::Continue),
        keyword!(TokenKind::Break),
        regex!(TokenKind::StringLiteral, &STRING_REGEX),
        regex!(TokenKind::IntLiteral, &INT_REGEX),
        regex!(TokenKind::FloatLiteral, &FLOAT_REGEX),
        regex!(TokenKind::BoolLiteral, &BOOL_REGEX),
        regex!(TokenKind::Identifier, &IDENTIFIER_REGEX),
    ]
}

pub(crate) fn get_unambiguous_token(char: char) -> Option<TokenKind> {
    match char {
        '(' => Some(TokenKind::ParenOpen),
        ')' => Some(TokenKind::ParenClose),
        '{' => Some(TokenKind::BraceOpen),
        '}' => Some(TokenKind::BraceClose),
        '[' => Some(TokenKind::SquareOpen),
        ']' => Some(TokenKind::SquareClose),
        ';' => Some(TokenKind::Semicolon),
        '.' => Some(TokenKind::Period),
        ',' => Some(TokenKind::Comma),
        '+' => Some(TokenKind::Plus),
        '*' => Some(TokenKind::Asterisk),
        _ => None,
    }
}
