use crate::error::print_error;
use crate::source::Source;
use crate::typechecker::CheckedFunctionItem;

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionErrorKind {
    MainFunctionNotFound,
    VariableDoesNotExist { name: String },
    VariableAlreadyExists { name: String },
    FunctionNotDefined { name: String },
    InvalidArgument { function: CheckedFunctionItem },
    InvalidNumberOfArguments { function: CheckedFunctionItem },
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
            ExecutionErrorKind::MainFunctionNotFound => {
                format!("Main function not found")
            }
            ExecutionErrorKind::VariableDoesNotExist { name } => {
                format!("Variable `{}` does not exist", name)
            }
            ExecutionErrorKind::VariableAlreadyExists { name } => {
                format!("Variable `{}` already exists", name)
            }
            ExecutionErrorKind::FunctionNotDefined { name } => {
                format!("Function `{}` is not defined", name)
            }
            ExecutionErrorKind::InvalidArgument { function } => {
                format!(
                    "Invalid argument for function `{}`",
                    function.definition.name
                )
            }
            ExecutionErrorKind::InvalidNumberOfArguments { function } => {
                format!(
                    "Invalid number of arguments for function `{}`",
                    function.definition.name
                )
            }
        };

        write!(f, "{}", str)
    }
}

pub type ExecutionResult<T> = Result<T, ExecutionError>;
