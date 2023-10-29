use crate::error::BauError;
use crate::parser::{ParsedFunctionArgument, ParsedFunctionItem, ParsedItem, TypeName};

#[derive(Debug, Clone, PartialEq)]
pub enum CheckedItem {
    Function(CheckedFunctionItem),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckedFunctionItem {
    pub name: String,
    pub arguments: Vec<CheckedFunctionArgument>,
    pub return_type: Type,
    pub body: Vec<CheckedStatement>,
    pub variables: Vec<CheckedVariable>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckedFunctionArgument {
    pub name: String,
    pub type_: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CheckedStatement {
    Let {
        name: String,
        type_: Type,
        initial_value: Option<CheckedExpression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CheckedExpression {
    Literal(CheckedLiteralExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CheckedLiteralExpression {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckedVariable {
    pub name: String,
    pub type_: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void,
    Integer,
    Float,
    String,
    Boolean,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Typechecker {
    errors: Vec<BauError>,
}

impl Typechecker {
    pub fn errors(&self) -> &[BauError] {
        &self.errors
    }

    pub fn new() -> Self {
        Self { errors: vec![] }
    }

    pub fn check_items(&mut self, items: &[ParsedItem]) -> Vec<CheckedItem> {
        let mut checked_items = vec![];
        for item in items.iter() {
            match item {
                ParsedItem::Function(function) => {
                    let checked_function = self.check_function(function);
                    checked_items.push(CheckedItem::Function(checked_function));
                }
            }
        }
        checked_items
    }

    fn check_function(&mut self, function: &ParsedFunctionItem) -> CheckedFunctionItem {
        let arguments = self.check_function_arguments(&function.arguments);

        let return_type = self.check_type(&function.return_type_name);

        let mut variables = vec![];
        for argument in arguments.iter() {
            variables.push(CheckedVariable {
                name: argument.name.clone(),
                type_: argument.type_.clone(),
            });
        }

        let body = vec![];

        CheckedFunctionItem {
            name: function.name.clone(),
            arguments,
            return_type,
            body,
            variables,
        }
    }

    fn check_function_arguments(
        &mut self,
        arguments: &[ParsedFunctionArgument],
    ) -> Vec<CheckedFunctionArgument> {
        let mut checked_arguments = vec![];
        for argument in arguments.iter() {
            let type_ = self.check_type(&argument.type_name);
            checked_arguments.push(CheckedFunctionArgument {
                name: argument.name.clone(),
                type_,
            });
        }
        checked_arguments
    }

    fn check_type(&mut self, type_name: &TypeName) -> Type {
        match type_name.name() {
            "void" => Type::Void,
            "int" => Type::Integer,
            "float" => Type::Float,
            "string" => Type::String,
            "bool" => Type::Boolean,
            _ => {
                self.errors.push(BauError::UnknownType {
                    token: type_name.token().clone(),
                    type_name: type_name.name().to_string(),
                });
                Type::Void
            }
        }
    }
}
