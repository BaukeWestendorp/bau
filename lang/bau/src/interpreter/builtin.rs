use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::typechecker::{CheckedExpression, CheckedFunctionDefinition};

use super::error::ExecutionResult;
use super::value::Value;
use super::Interpreter;

macro_rules! function_definition {
    (fn $name:ident($($arg_name:ident: $arg_type:ty),*) -> $return_type:ty) => {
        CheckedFunctionDefinition {
            name: stringify!($name).to_string(),
            parameters: vec![
                $(
                    crate::typechecker::CheckedFunctionParameter     {
                        name: stringify!($arg_name).to_string(),
                        type_: match stringify!($arg_type) {
                            "string" => crate::typechecker::Type::String,
                            "int" => crate::typechecker::Type::Integer,
                            "float" => crate::typechecker::Type::Float,
                            "bool" => crate::typechecker::Type::Boolean,
                            _ => panic!("Unknown type"),
                        },
                    }
                ),*
            ],
            return_type: match stringify!($return_type) {
                "void" => crate::typechecker::Type::Void,
                "string" => crate::typechecker::Type::String,
                "int" => crate::typechecker::Type::Integer,
                "float" => crate::typechecker::Type::Float,
                "bool" => crate::typechecker::Type::Boolean,
                _ => panic!("Unknown type"),
            },
        }
    };
}

lazy_static! {
    pub static ref BUILTIN_FUNCTIONS: HashMap<&'static str, CheckedFunctionDefinition> = {
        let mut map = HashMap::new();
        map.insert(
            "print",
            function_definition!(fn print(value: string) -> void),
        );
        map
    };
}

pub fn evaluate_builtin_function(
    interpreter: &mut Interpreter,
    name: &str,
    arguments: &Vec<CheckedExpression>,
) -> ExecutionResult<Option<Value>> {
    match name {
        "print" => {
            let value = interpreter.evaluate_expression(&arguments[0])?;
            println!("{:?}", value);
            Ok(None)
        }
        _ => panic!("Unknown builtin function"),
    }
}
