use crate::error::print_error;
use crate::source::Source;

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionErrorKind {
    Panic { message: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionError {
    kind: ExecutionErrorKind,
}

impl ExecutionError {
    pub fn new(kind: ExecutionErrorKind) -> Self {
        Self { kind }
    }

    pub fn print(&self, source: &Source) {
        print_error(source, None, &self.to_string());
    }
}

impl std::error::Error for ExecutionError {}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = match &self.kind {
            ExecutionErrorKind::Panic { message } => format!("Panic: {}", message),
        };

        write!(f, "{}", str)
    }
}

pub type ExecutionResult<T> = Result<T, ExecutionError>;
