use crate::error::BauError;
use crate::parser::{
    ParsedExpression, ParsedExpressionKind, ParsedFunctionArgument, ParsedFunctionItem, ParsedItem,
    ParsedLiteralExpression, ParsedStatement, TypeName,
};

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
        initial_value: CheckedExpression,
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

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Void => "void",
            Self::Integer => "int",
            Self::Float => "float",
            Self::String => "string",
            Self::Boolean => "bool",
        };

        write!(f, "{}", str)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    variables: Vec<CheckedVariable>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Typechecker {
    errors: Vec<BauError>,
    scope_stack: Vec<Scope>,
}

impl Typechecker {
    pub fn errors(&self) -> &[BauError] {
        &self.errors
    }

    pub fn new() -> Self {
        Self {
            errors: vec![],
            scope_stack: vec![],
        }
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
        self.push_scope();

        let arguments = self.check_function_arguments(&function.arguments);

        let return_type = self.check_type(&function.return_type_name);

        for argument in arguments.iter() {
            self.register_var_in_current_scope(CheckedVariable {
                name: argument.name.clone(),
                type_: argument.type_.clone(),
            });
        }

        let body = self.check_function_body(&function.body);

        self.pop_scope();

        CheckedFunctionItem {
            name: function.name.clone(),
            arguments,
            return_type,
            body,
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

    fn check_function_body(&mut self, body: &[ParsedStatement]) -> Vec<CheckedStatement> {
        let mut checked_body = vec![];
        for statement in body.iter() {
            let checked_statement = self.check_statement(statement);
            checked_body.push(checked_statement);
        }
        checked_body
    }

    fn check_statement(&mut self, statement: &ParsedStatement) -> CheckedStatement {
        match statement {
            ParsedStatement::Let { .. } => self.check_let_statement(statement),
        }
    }

    fn check_let_statement(&mut self, statement: &ParsedStatement) -> CheckedStatement {
        match statement {
            ParsedStatement::Let {
                name,
                type_name,
                initial_value,
            } => {
                if self.variable_exists(name.name()) {
                    self.errors.push(BauError::VariableAlreadyExists {
                        token: name.token().clone(),
                        name: name.name().to_string(),
                    });
                }

                let type_ = self.check_type(type_name);
                let (checked_initial_value, initial_value_type) =
                    self.check_expression(initial_value);

                if type_ != initial_value_type {
                    self.errors.push(BauError::TypeMismatch {
                        token: initial_value.token().clone(),
                        expected: type_.clone(),
                        actual: initial_value_type,
                    });
                }

                self.register_var_in_current_scope(CheckedVariable {
                    name: name.name().to_string(),
                    type_: type_.clone(),
                });

                CheckedStatement::Let {
                    name: name.name().to_string(),
                    type_,
                    initial_value: checked_initial_value,
                }
            }
        }
    }

    fn check_expression(&mut self, expression: &ParsedExpression) -> (CheckedExpression, Type) {
        match expression.kind() {
            ParsedExpressionKind::Literal(literal) => {
                let checked_literal = self.check_literal_expression(literal);
                (
                    CheckedExpression::Literal(checked_literal.0),
                    checked_literal.1,
                )
            }
        }
    }

    fn check_literal_expression(
        &mut self,
        expression: &ParsedLiteralExpression,
    ) -> (CheckedLiteralExpression, Type) {
        match expression {
            ParsedLiteralExpression::Integer(value) => {
                (CheckedLiteralExpression::Integer(*value), Type::Integer)
            }
            ParsedLiteralExpression::Float(value) => {
                (CheckedLiteralExpression::Float(*value), Type::Float)
            }
            ParsedLiteralExpression::String(value) => (
                CheckedLiteralExpression::String(value.clone()),
                Type::String,
            ),
            ParsedLiteralExpression::Boolean(value) => {
                (CheckedLiteralExpression::Boolean(*value), Type::Boolean)
            }
        }
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

    fn push_scope(&mut self) {
        self.scope_stack.push(Scope { variables: vec![] });
    }

    fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn register_var_in_current_scope(&mut self, variable: CheckedVariable) {
        let current_scope = self.scope_stack.last_mut().unwrap();
        current_scope.variables.push(variable);
    }

    fn variable_exists(&mut self, name: &str) -> bool {
        for scope in self.scope_stack.iter().rev() {
            for variable in scope.variables.iter() {
                if variable.name == name {
                    return true;
                }
            }
        }
        false
    }
}
