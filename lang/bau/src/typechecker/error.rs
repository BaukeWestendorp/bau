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
    InvalidVoidExpression,
    MainFunctionNotDefined,
    MethodNotDefined {
        type_: Type,
        method_name: String,
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
                "Cannot return a value in a void function".to_string()
            }
            TypecheckerErrorKind::ExpectedReturnValue => "Expected a return value".to_string(),
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
            TypecheckerErrorKind::InvalidVoidExpression => {
                "Cannot use void expression in this context".to_string()
            }
            TypecheckerErrorKind::MainFunctionNotDefined => {
                "Main function is not defined".to_string()
            }
            TypecheckerErrorKind::MethodNotDefined { type_, method_name } => {
                format!(
                    "Method `{}` is not defined for type `{}`",
                    method_name, type_
                )
            }
        };

        write!(f, "{}", str)
    }
}

pub type TypecheckerResult<T> = Result<T, TypecheckerError>;
