use crate::parser::{
    Identifier, ParsedExpression, ParsedExpressionKind, ParsedFunctionParameter, ParsedItem,
    ParsedItemKind, ParsedLiteralExpression, ParsedStatement, ParsedStatementKind, TypeName,
};

use crate::source::CodeRange;

pub mod error;

pub use error::TypecheckerError;

use self::error::TypecheckerErrorKind;

#[derive(Debug, Clone, PartialEq)]
pub enum CheckedItemKind {
    Function(CheckedFunctionItem),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckedItem {
    kind: CheckedItemKind,
    range: CodeRange,
}

impl CheckedItem {
    pub fn kind(&self) -> &CheckedItemKind {
        &self.kind
    }

    pub fn range(&self) -> &CodeRange {
        &self.range
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckedFunctionItem {
    pub name: String,
    pub parameters: Vec<CheckedFunctionParameter>,
    pub return_type: Type,
    pub body: Vec<CheckedStatement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckedFunctionParameter {
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
    Variable(CheckedVariable),
}

impl CheckedExpression {
    pub fn type_(&self) -> Type {
        match self {
            Self::Literal(literal) => match literal {
                CheckedLiteralExpression::Integer(_) => Type::Integer,
                CheckedLiteralExpression::Float(_) => Type::Float,
                CheckedLiteralExpression::String(_) => Type::String,
                CheckedLiteralExpression::Boolean(_) => Type::Boolean,
            },
            Self::Variable(variable) => variable.type_.clone(),
        }
    }
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
    errors: Vec<TypecheckerError>,
    scope_stack: Vec<Scope>,
}

impl Typechecker {
    pub fn errors(&self) -> &[TypecheckerError] {
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

        let parameters = self.check_function_parameters(&function.parameters);

        let return_type = self.check_type(&function.return_type_name);

        for parameter in parameters.iter() {
            self.register_var_in_current_scope(CheckedVariable {
                name: parameter.name.clone(),
                type_: parameter.type_.clone(),
            });
        }

        let body = self.check_function_body(&function.body, &return_type);

        let return_statement = body.iter().find(|statement| match statement.kind() {
            CheckedStatementKind::Return { .. } => true,
            _ => false,
        });

        if return_type == Type::Void && return_statement.is_some() {
            self.errors.push(TypecheckerError::new(
                TypecheckerErrorKind::ReturnValueInVoidFunction,
                return_statement.unwrap().range().clone(),
            ));
        } else if return_type != Type::Void && return_statement.is_none() {
            self.errors.push(TypecheckerError::new(
                TypecheckerErrorKind::ExpectedReturnValue,
                function_item.range().clone(),
            ));
        }

        self.pop_scope();

        CheckedFunctionItem {
            name: function.name.clone(),
            parameters,
            return_type,
            body,
        }
    }

    fn check_function_parameters(
        &mut self,
        parameters: &[ParsedFunctionParameter],
    ) -> Vec<CheckedFunctionParameter> {
        let mut checked_parameters = vec![];
        for parameter in parameters.iter() {
            let type_ = self.check_type(&parameter.type_name);
            checked_parameters.push(CheckedFunctionParameter {
                name: parameter.name.clone(),
                type_,
            });
        }
        checked_parameters
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
                    self.errors.push(TypecheckerError::new(
                        TypecheckerErrorKind::VariableAlreadyExists {
                            name: name.name().to_string(),
                        },
                        name.token().range.clone(),
                    ));
                }

                let type_ = self.check_type(type_name);
                let checked_initial_value = self.check_expression(initial_value);

                if type_ != checked_initial_value.type_() {
                    self.errors.push(TypecheckerError::new(
                        TypecheckerErrorKind::TypeMismatch {
                            expected: type_.clone(),
                            actual: checked_initial_value.type_(),
                        },
                        type_name.token().range.clone(),
                    ));
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
                    self.errors.push(TypecheckerError::new(
                        TypecheckerErrorKind::ReturnValueInVoidFunction,
                        statement.range().clone(),
                    ));
                    return CheckedStatement {
                        kind: CheckedStatementKind::Return { value: None },
                        range: statement.range().clone(),
                    };
                } else if parent_function_return_type != &Type::Void && value.is_none() {
                    self.errors.push(TypecheckerError::new(
                        TypecheckerErrorKind::ExpectedReturnValue,
                        statement.range().clone(),
                    ));
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
                    let checked_value = self.check_expression(&value);

                    if parent_function_return_type != &checked_value.type_() {
                        self.errors.push(TypecheckerError::new(
                            TypecheckerErrorKind::TypeMismatch {
                                expected: parent_function_return_type.clone(),
                                actual: checked_value.type_(),
                            },
                            value.token().range.clone(),
                        ));
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

    fn check_expression(&mut self, expression: &ParsedExpression) -> CheckedExpression {
        match expression.kind() {
            ParsedExpressionKind::Literal(literal) => {
                let checked_literal = self.check_literal_expression(literal);
                CheckedExpression::Literal(checked_literal)
            }
            ParsedExpressionKind::Variable(ident) => {
                if !self.variable_exists(ident.name()) {
                    self.errors.push(TypecheckerError::new(
                        TypecheckerErrorKind::VariableDoesNotExist {
                            name: ident.name().to_string(),
                        },
                        ident.token().range.clone(),
                    ));
                }

                let checked_variable = self.check_variable_expression(ident);
                CheckedExpression::Variable(checked_variable)
            }
        }
    }

    fn check_literal_expression(
        &mut self,
        expression: &ParsedLiteralExpression,
    ) -> CheckedLiteralExpression {
        match expression {
            ParsedLiteralExpression::Integer(value) => CheckedLiteralExpression::Integer(*value),
            ParsedLiteralExpression::Float(value) => CheckedLiteralExpression::Float(*value),
            ParsedLiteralExpression::String(value) => {
                CheckedLiteralExpression::String(value.clone())
            }
            ParsedLiteralExpression::Boolean(value) => CheckedLiteralExpression::Boolean(*value),
        }
    }

    fn check_variable_expression(&mut self, ident: &Identifier) -> CheckedVariable {
        let variable = self.get_variable_by_name(ident.name());
        if let Some(variable) = variable {
            variable
        } else {
            self.register_var_in_current_scope(CheckedVariable {
                name: ident.name().to_string(),
                type_: Type::Void,
            });
            CheckedVariable {
                name: ident.name().to_string(),
                type_: Type::Void,
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
                self.errors.push(TypecheckerError::new(
                    TypecheckerErrorKind::UnknownType {
                        type_name: type_name.name().to_string(),
                    },
                    type_name.token().range.clone(),
                ));
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
