use crate::error::BauError;
use crate::parser::{
    Identifier, ParsedExpression, ParsedExpressionKind, ParsedFunctionArgument, ParsedItem,
    ParsedItemKind, ParsedLiteralExpression, ParsedStatement, ParsedStatementKind, TypeName,
};
use crate::source::CodeRange;

#[derive(Debug, Clone, PartialEq)]
pub enum CheckedItemKind {
    Function(CheckedFunctionItem),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckedItem {
    kind: CheckedItemKind,
    range: CodeRange,
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
pub enum CheckedStatementKind {
    Let {
        name: String,
        type_: Type,
        initial_value: CheckedExpression,
    },
    Return {
        value: Option<CheckedExpression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckedStatement {
    kind: CheckedStatementKind,
    range: CodeRange,
}

impl CheckedStatement {
    pub fn kind(&self) -> &CheckedStatementKind {
        &self.kind
    }

    pub fn range(&self) -> &CodeRange {
        &self.range
    }
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
pub struct CheckedVariableExpression {
    pub variable: CheckedVariable,
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
            match item.kind() {
                ParsedItemKind::Function(_) => {
                    checked_items.push(CheckedItem {
                        kind: CheckedItemKind::Function(self.check_function_item(item)),
                        range: item.range().clone(),
                    });
                }
            }
        }
        checked_items
    }

    fn check_function_item(&mut self, function_item: &ParsedItem) -> CheckedFunctionItem {
        let function = match function_item.kind() {
            ParsedItemKind::Function(function) => function,
        };

        self.push_scope();

        let arguments = self.check_function_arguments(&function.arguments);

        let return_type = self.check_type(&function.return_type_name);

        for argument in arguments.iter() {
            self.register_var_in_current_scope(CheckedVariable {
                name: argument.name.clone(),
                type_: argument.type_.clone(),
            });
        }

        let body = self.check_function_body(&function.body, &return_type);

        let return_statement = body.iter().find(|statement| match statement.kind() {
            CheckedStatementKind::Return { .. } => true,
            _ => false,
        });

        if return_type == Type::Void && return_statement.is_some() {
            self.errors.push(BauError::ReturnValueInVoidFunction {
                range: return_statement.unwrap().range().clone(),
            });
        } else if return_type != Type::Void && return_statement.is_none() {
            self.errors.push(BauError::ExpectedReturnValue {
                range: function_item.range().clone(),
            });
        }

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

    fn check_function_body(
        &mut self,
        body: &[ParsedStatement],
        parent_function_return_type: &Type,
    ) -> Vec<CheckedStatement> {
        let mut checked_body = vec![];
        for statement in body.iter() {
            let checked_statement = self.check_statement(statement, parent_function_return_type);
            checked_body.push(checked_statement);
        }
        checked_body
    }

    fn check_statement(
        &mut self,
        statement: &ParsedStatement,
        parent_function_return_type: &Type,
    ) -> CheckedStatement {
        match statement.kind() {
            ParsedStatementKind::Let { .. } => self.check_let_statement(statement),
            ParsedStatementKind::Return { .. } => {
                self.check_return_statement(statement, parent_function_return_type)
            }
        }
    }

    fn check_let_statement(&mut self, statement: &ParsedStatement) -> CheckedStatement {
        match statement.kind() {
            ParsedStatementKind::Let {
                name,
                type_name,
                initial_value,
            } => {
                if self.variable_exists(name.name()) {
                    self.errors.push(BauError::VariableAlreadyExists {
                        range: name.token().range.clone(),
                        name: name.name().to_string(),
                    });
                }

                let type_ = self.check_type(type_name);
                let (checked_initial_value, initial_value_type) =
                    self.check_expression(initial_value);

                if type_ != initial_value_type {
                    self.errors.push(BauError::TypeMismatch {
                        range: type_name.token().range.clone(),
                        expected: type_.clone(),
                        actual: initial_value_type,
                    });
                }

                self.register_var_in_current_scope(CheckedVariable {
                    name: name.name().to_string(),
                    type_: type_.clone(),
                });

                CheckedStatement {
                    kind: CheckedStatementKind::Let {
                        name: name.name().to_string(),
                        type_,
                        initial_value: checked_initial_value,
                    },
                    range: statement.range().clone(),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    fn check_return_statement(
        &mut self,
        statement: &ParsedStatement,
        parent_function_return_type: &Type,
    ) -> CheckedStatement {
        match statement.kind() {
            ParsedStatementKind::Return { value } => {
                if parent_function_return_type == &Type::Void && value.is_some() {
                    self.errors.push(BauError::ReturnValueInVoidFunction {
                        range: statement.range().clone(),
                    });
                    return CheckedStatement {
                        kind: CheckedStatementKind::Return { value: None },
                        range: statement.range().clone(),
                    };
                } else if parent_function_return_type != &Type::Void && value.is_none() {
                    self.errors.push(BauError::ExpectedReturnValue {
                        range: statement.range().clone(),
                    });
                    return CheckedStatement {
                        kind: CheckedStatementKind::Return { value: None },
                        range: statement.range().clone(),
                    };
                } else if parent_function_return_type == &Type::Void && value.is_none() {
                    return CheckedStatement {
                        kind: CheckedStatementKind::Return { value: None },
                        range: statement.range().clone(),
                    };
                } else {
                    let value = value.clone().unwrap();
                    let (checked_value, value_type) = self.check_expression(&value);

                    if parent_function_return_type != &value_type {
                        self.errors.push(BauError::TypeMismatch {
                            range: value.token().range.clone(),
                            expected: parent_function_return_type.clone(),
                            actual: value_type,
                        });
                    }

                    return CheckedStatement {
                        kind: CheckedStatementKind::Return {
                            value: Some(checked_value),
                        },
                        range: statement.range().clone(),
                    };
                }
            }
            _ => panic!("Expected return statement"),
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
            ParsedExpressionKind::Variable(ident) => {
                let checked_variable = self.check_variable_expression(ident);
                (
                    CheckedExpression::Literal(CheckedLiteralExpression::Integer(0)),
                    checked_variable.variable.type_.clone(),
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

    fn check_variable_expression(&mut self, ident: &Identifier) -> CheckedVariableExpression {
        let variable = self.get_variable_by_name(ident.name());
        if let Some(variable) = variable {
            CheckedVariableExpression { variable }
        } else {
            self.errors.push(BauError::VariableDoesNotExist {
                range: ident.token().range.clone(),
                name: ident.name().to_string(),
            });
            CheckedVariableExpression {
                variable: CheckedVariable {
                    name: ident.name().to_string(),
                    type_: Type::Void,
                },
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
                    range: type_name.token().range.clone(),
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
        self.get_variable_by_name(name).is_some()
    }

    fn get_variable_by_name(&mut self, name: &str) -> Option<CheckedVariable> {
        for scope in self.scope_stack.iter().rev() {
            for variable in scope.variables.iter() {
                if variable.name == name {
                    return Some(variable.clone());
                }
            }
        }
        None
    }
}
