use crate::error::print_error;
use crate::source::{CodeRange, Source};
use crate::tokenizer::token::TokenKind;

use super::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum TypecheckerErrorKind {
    UnknownType {
        type_name: String,
    },
    TypeMismatch {
        expected: Type,
        actual: Type,
    },
    VariableAlreadyDefined {
        name: String,
    },
    VariableNotDefined {
        name: String,
    },
    FunctionNotDefined {
        name: String,
    },
    ReturnValueInVoidFunction,
    ExpectedReturnValue,
    IncompatibleInfixSides {
        left: Type,
        operator: TokenKind,
        right: Type,
    },
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
            TypecheckerErrorKind::IncompatibleInfixSides {
                left,
                operator,
                right,
            } => match operator {
                TokenKind::Plus | TokenKind::Minus | TokenKind::Asterisk | TokenKind::Slash => {
                    format!("The `{}` operator can only be used between two floats or two ints, but found `{}` and `{}`", operator, left, right)
                }
                TokenKind::EqualsEquals
                | TokenKind::ExclamationMarkEquals
                | TokenKind::LessThan
                | TokenKind::GreaterThan
                | TokenKind::LessThanEquals
                | TokenKind::GreaterThanEquals => {
                    format!("The `{}` operator can only be used between two floats or two ints, but found `{}` and `{}`", operator, left, right)
                }
                _ => {
                    format!(
                        "Invalid types for `{}` operator: `{}` and `{}`",
                        operator, left, right
                    )
                }
            },
        };

        write!(f, "{}", str)
    }
}

pub type TypecheckerResult<T> = Result<T, TypecheckerError>;
