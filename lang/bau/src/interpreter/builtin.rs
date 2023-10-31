use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::typechecker::{CheckedExpression, CheckedFunctionDefinition};

use super::error::ExecutionResult;
use super::value::Value;
use super::Interpreter;

macro_rules! type_name_to_type {
    ($type_name:ident) => {
        match stringify!($type_name) {
            "void" => crate::typechecker::Type::Void,
            "string" => crate::typechecker::Type::String,
            "int" => crate::typechecker::Type::Integer,
            "float" => crate::typechecker::Type::Float,
            "bool" => crate::typechecker::Type::Boolean,
            _ => panic!("Unknown type: `{}`", stringify!($type_name)),
        }
    };
}

macro_rules! function_definition {
    (fn $name:ident($($arg_name:ident: $arg_type:ident),*) -> $return_type:ident) => {
        CheckedFunctionDefinition {
            name: stringify!($name).to_string(),
            parameters: vec![
                $(
                    crate::typechecker::CheckedFunctionParameter     {
                        name: stringify!($arg_name).to_string(),
                        type_: type_name_to_type!($return_type),
                    }
                ),*
            ],
            return_type: type_name_to_type!($return_type),
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
    arguments: &[CheckedExpression],
) -> ExecutionResult<Option<Value>> {
    let builtin_definition = BUILTIN_FUNCTIONS.get(name).unwrap();

    assert_eq!(
        builtin_definition.parameters.len(),
        arguments.len(),
        "Typechecker should have checked argument counts. Expected {} arguments, but found {}",
        builtin_definition.parameters.len(),
        arguments.len()
    );

    match name {
        "print" => {
            match interpreter.evaluate_expression(&arguments[0])? {
                Some(value) => println!("{}", value),
                None => println!("None"),
            }
            Ok(None)
        }
        _ => panic!("Unknown builtin function `{}`", name),
    }
}
