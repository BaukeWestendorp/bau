use crate::error::print_error;
use crate::source::Source;
use crate::typechecker::CheckedFunctionItem;

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionErrorKind {
    MainFunctionNotFound,
    VariableDoesNotExist {
        name: String,
    },
    VariableAlreadyExists {
        name: String,
    },
    FunctionNotDefined {
        name: String,
    },
    InvalidArgument {
        function: CheckedFunctionItem,
    },
    InvalidNumberOfArguments {
        name: String,
        expected_number: usize,
        found_number: usize,
    },
    PrefixWithInvalidType,
    InfixWithVoidSide,
    InfixWithInvalidTypes,
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
            ExecutionErrorKind::MainFunctionNotFound => "Main function not found".to_string(),
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
            ExecutionErrorKind::InvalidNumberOfArguments {
                name,
                expected_number,
                found_number,
            } => {
                format!(
                    "Invalid number of arguments for function `{}`: expected {}, found {}",
                    name, expected_number, found_number
                )
            }
            ExecutionErrorKind::PrefixWithInvalidType => {
                "Prefix operator has invalid type".to_string()
            }
            ExecutionErrorKind::InfixWithVoidSide => {
                "Infix operator can't be used with a side of type `void`".to_string()
            }
            ExecutionErrorKind::InfixWithInvalidTypes => {
                "Infix operator has invalid types".to_string()
            }
        };

        write!(f, "{}", str)
    }
}

pub type ExecutionResult<T> = Result<T, ExecutionError>;
