#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum BauError {
    ParseError {
        line: usize,
        column: usize,
        message: String,
    },
}

pub type BauResult<T> = Result<T, BauError>;
