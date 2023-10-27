use crate::tokenizer::Token;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum BauError {
    #[error("Unexpected end of file")]
    UnexpectedEndOfFile(Token),
    #[error("Unexpected token: {0:?}")]
    UnexpectedToken(Token),
}

pub type BauResult<T> = Result<T, BauError>;
