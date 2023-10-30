use crate::error::print_error;
use crate::source::{CodeRange, Source};

use super::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum TypecheckerErrorKind {
    UnknownType { type_name: String },
    TypeMismatch { expected: Type, actual: Type },
    VariableAlreadyDefined { name: String },
    VariableNotDefined { name: String },
    FunctionNotDefined { name: String },
    ReturnValueInVoidFunction,
    ExpectedReturnValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypecheckerError {
    kind: TypecheckerErrorKind,
    range: CodeRange,
}

impl TypecheckerError {
    pub fn new(kind: TypecheckerErrorKind, range: CodeRange) -> Self {
        Self { kind, range }
    }

    pub fn print(&self, source: &Source) {
        print_error(source, Some(&self.range), &self.to_string());
    }
}

impl std::error::Error for TypecheckerError {}

impl std::fmt::Display for TypecheckerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str = match &self.kind {
            TypecheckerErrorKind::UnknownType { type_name } => {
                format!("Unknown type `{}`", type_name)
            }
            TypecheckerErrorKind::TypeMismatch { expected, actual } => {
                format!(
                    "Expected type `{}`, but found `{}` instead",
                    expected, actual
                )
            }
            TypecheckerErrorKind::VariableAlreadyDefined { name } => {
                format!("Variable `{}` is already defined", name)
            }
            TypecheckerErrorKind::VariableNotDefined { name } => {
                format!("Variable `{}` is not defined", name)
            }
            TypecheckerErrorKind::FunctionNotDefined { name } => {
                format!("Function `{}` is not defined", name)
            }
            TypecheckerErrorKind::ReturnValueInVoidFunction => {
                format!("Cannot return a value in a void function")
            }
            TypecheckerErrorKind::ExpectedReturnValue => {
                format!("Expected a return value")
            }
        };

        write!(f, "{}", str)
    }
}

pub type TypecheckerResult<T> = Result<T, TypecheckerError>;
