use std::collections::HashMap;

use crate::parser::{
    Identifier, ParsedExpression, ParsedExpressionKind, ParsedFunctionParameter, ParsedItem,
    ParsedItemKind, ParsedLiteralExpression, ParsedStatement, ParsedStatementKind, TypeName,
};

use crate::source::CodeRange;

pub mod error;

pub use error::TypecheckerError;

use self::error::{TypecheckerErrorKind, TypecheckerResult};

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
    pub definition: CheckedFunctionDefinition,
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
    FunctionCall {
        name: Identifier,
        arguments: Vec<CheckedExpression>,
    },
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
pub struct CheckedFunctionDefinition {
    pub name: String,
    pub parameters: Vec<CheckedFunctionParameter>,
    pub return_type: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    variables: Vec<CheckedVariable>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Typechecker {
    errors: Vec<TypecheckerError>,
    scope_stack: Vec<Scope>,
    functions: HashMap<String, CheckedFunctionDefinition>,
}

impl Typechecker {
    pub fn errors(&self) -> &[TypecheckerError] {
        &self.errors
    }

    pub fn new() -> Self {
        Self {
            errors: vec![],
            scope_stack: vec![],
            functions: HashMap::new(),
        }
    }

    pub fn check_items(&mut self, items: &[ParsedItem]) -> Vec<CheckedItem> {
        // First let's find all function definitions
        for item in items.iter() {
            match item.kind() {
                ParsedItemKind::Function(_) => {
                    let function_definition = match self.check_function_definition(item) {
                        Ok(function_definition) => function_definition,
                        Err(error) => {
                            self.errors.push(error);
                            continue;
                        }
                    };
                    self.register_function(function_definition);
                }
            }
        }

        // FIXME: We might be able to only check function definitions once somehow.
        //        Currently we still check the function definitions here again.
        let mut checked_items = vec![];
        for item in items.iter() {
            match item.kind() {
                ParsedItemKind::Function(_) => {
                    let function = match self.check_function_item(item) {
                        Ok(function) => function,
                        Err(error) => {
                            self.errors.push(error);
                            continue;
                        }
                    };
                    checked_items.push(CheckedItem {
                        kind: CheckedItemKind::Function(function),
                        range: item.range().clone(),
                    });
                }
            }
        }
        checked_items
    }

    fn check_function_item(
        &mut self,
        function_item: &ParsedItem,
    ) -> TypecheckerResult<CheckedFunctionItem> {
        let definition = self.check_function_definition(function_item)?;

        let function = match function_item.kind() {
            ParsedItemKind::Function(function) => function,
        };

        self.push_scope();

        let body = self.check_function_body(&function.body, &definition.return_type)?;

        let return_statement = body.iter().find(|statement| match statement.kind() {
            CheckedStatementKind::Return { .. } => true,
            _ => false,
        });

        if definition.return_type == Type::Void && return_statement.is_some() {
            self.pop_scope();
            return Err(TypecheckerError::new(
                TypecheckerErrorKind::ReturnValueInVoidFunction,
                return_statement.unwrap().range().clone(),
            ));
        } else if definition.return_type != Type::Void && return_statement.is_none() {
            self.pop_scope();
            return Err(TypecheckerError::new(
                TypecheckerErrorKind::ExpectedReturnValue,
                function_item.range().clone(),
            ));
        }

        self.pop_scope();

        Ok(CheckedFunctionItem { definition, body })
    }

    fn check_function_definition(
        &mut self,
        function_item: &ParsedItem,
    ) -> TypecheckerResult<CheckedFunctionDefinition> {
        let function = match function_item.kind() {
            ParsedItemKind::Function(function) => function,
        };

        let parameters = self.check_function_parameters(&function.parameters)?;

        let return_type = self.check_type(&function.return_type_name)?;

        for parameter in parameters.iter() {
            self.register_var_in_current_scope(CheckedVariable {
                name: parameter.name.clone(),
                type_: parameter.type_.clone(),
            });
        }

        Ok(CheckedFunctionDefinition {
            name: function.name.clone(),
            parameters,
            return_type,
        })
    }

    fn check_function_parameters(
        &mut self,
        parameters: &[ParsedFunctionParameter],
    ) -> TypecheckerResult<Vec<CheckedFunctionParameter>> {
        let mut checked_parameters = vec![];
        for parameter in parameters.iter() {
            let type_ = self.check_type(&parameter.type_name)?;
            checked_parameters.push(CheckedFunctionParameter {
                name: parameter.name.clone(),
                type_,
            });
        }
        Ok(checked_parameters)
    }

    fn check_function_body(
        &mut self,
        body: &[ParsedStatement],
        parent_function_return_type: &Type,
    ) -> TypecheckerResult<Vec<CheckedStatement>> {
        let mut checked_body = vec![];
        for statement in body.iter() {
            let checked_statement = self.check_statement(statement, parent_function_return_type)?;
            checked_body.push(checked_statement);
        }
        Ok(checked_body)
    }

    fn check_statement(
        &mut self,
        statement: &ParsedStatement,
        parent_function_return_type: &Type,
    ) -> TypecheckerResult<CheckedStatement> {
        match statement.kind() {
            ParsedStatementKind::Let { .. } => self.check_let_statement(statement),
            ParsedStatementKind::Return { .. } => {
                self.check_return_statement(statement, parent_function_return_type)
            }
        }
    }

    fn check_let_statement(
        &mut self,
        statement: &ParsedStatement,
    ) -> TypecheckerResult<CheckedStatement> {
        match statement.kind() {
            ParsedStatementKind::Let {
                name,
                type_name,
                initial_value,
            } => {
                if self.variable_exists(name.name()) {
                    return Err(TypecheckerError::new(
                        TypecheckerErrorKind::VariableAlreadyDefined {
                            name: name.name().to_string(),
                        },
                        name.token().range.clone(),
                    ));
                }

                let type_ = self.check_type(type_name)?;
                let checked_initial_value = self.check_expression(initial_value)?;

                if type_ != self.expression_type(&checked_initial_value)? {
                    return Err(TypecheckerError::new(
                        TypecheckerErrorKind::TypeMismatch {
                            expected: type_.clone(),
                            actual: self.expression_type(&checked_initial_value)?,
                        },
                        type_name.token().range.clone(),
                    ));
                }

                self.register_var_in_current_scope(CheckedVariable {
                    name: name.name().to_string(),
                    type_: type_.clone(),
                });

                Ok(CheckedStatement {
                    kind: CheckedStatementKind::Let {
                        name: name.name().to_string(),
                        type_,
                        initial_value: checked_initial_value,
                    },
                    range: statement.range().clone(),
                })
            }
            _ => panic!("Expected let statement"),
        }
    }

    fn check_return_statement(
        &mut self,
        statement: &ParsedStatement,
        parent_function_return_type: &Type,
    ) -> TypecheckerResult<CheckedStatement> {
        match statement.kind() {
            ParsedStatementKind::Return { value } => {
                if parent_function_return_type == &Type::Void && value.is_some() {
                    Err(TypecheckerError::new(
                        TypecheckerErrorKind::ReturnValueInVoidFunction,
                        statement.range().clone(),
                    ))
                } else if parent_function_return_type != &Type::Void && value.is_none() {
                    Err(TypecheckerError::new(
                        TypecheckerErrorKind::ExpectedReturnValue,
                        statement.range().clone(),
                    ))
                } else if parent_function_return_type == &Type::Void && value.is_none() {
                    Ok(CheckedStatement {
                        kind: CheckedStatementKind::Return { value: None },
                        range: statement.range().clone(),
                    })
                } else {
                    let value = value.clone().unwrap();
                    let checked_value = self.check_expression(&value)?;

                    if parent_function_return_type != &self.expression_type(&checked_value)? {
                        return Err(TypecheckerError::new(
                            TypecheckerErrorKind::TypeMismatch {
                                expected: parent_function_return_type.clone(),
                                actual: self.expression_type(&checked_value)?,
                            },
                            value.token().range.clone(),
                        ));
                    }

                    Ok(CheckedStatement {
                        kind: CheckedStatementKind::Return {
                            value: Some(checked_value),
                        },
                        range: statement.range().clone(),
                    })
                }
            }
            _ => panic!("Expected return statement"),
        }
    }

    fn check_expression(
        &mut self,
        expression: &ParsedExpression,
    ) -> TypecheckerResult<CheckedExpression> {
        match expression.kind() {
            ParsedExpressionKind::Literal(literal) => {
                let checked_literal = self.check_literal_expression(literal);
                Ok(CheckedExpression::Literal(checked_literal))
            }
            ParsedExpressionKind::Variable(ident) => {
                if !self.variable_exists(ident.name()) {
                    return Err(TypecheckerError::new(
                        TypecheckerErrorKind::VariableNotDefined {
                            name: ident.name().to_string(),
                        },
                        ident.token().range.clone(),
                    ));
                }

                let checked_variable = self.check_variable_expression(ident)?;
                Ok(CheckedExpression::Variable(checked_variable))
            }
            ParsedExpressionKind::FunctionCall { name, arguments } => {
                // FIXME: Implement arguments
                Ok(CheckedExpression::FunctionCall {
                    name: Identifier::new(name.name().to_string(), name.token().clone()),
                    arguments: vec![],
                })
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

    fn check_variable_expression(
        &mut self,
        ident: &Identifier,
    ) -> TypecheckerResult<CheckedVariable> {
        let variable = self.get_variable_by_name(ident.name());
        if let Some(variable) = variable {
            Ok(variable)
        } else {
            Err(TypecheckerError::new(
                TypecheckerErrorKind::VariableNotDefined {
                    name: ident.name().to_string(),
                },
                ident.token().range.clone(),
            ))
        }
    }

    fn check_type(&mut self, type_name: &TypeName) -> TypecheckerResult<Type> {
        match type_name.name() {
            "void" => Ok(Type::Void),
            "int" => Ok(Type::Integer),
            "float" => Ok(Type::Float),
            "string" => Ok(Type::String),
            "bool" => Ok(Type::Boolean),
            _ => Err(TypecheckerError::new(
                TypecheckerErrorKind::UnknownType {
                    type_name: type_name.name().to_string(),
                },
                type_name.token().range.clone(),
            )),
        }
    }

    fn expression_type(&self, expression: &CheckedExpression) -> TypecheckerResult<Type> {
        match expression {
            CheckedExpression::Literal(literal) => match literal {
                CheckedLiteralExpression::Integer(_) => Ok(Type::Integer),
                CheckedLiteralExpression::Float(_) => Ok(Type::Float),
                CheckedLiteralExpression::String(_) => Ok(Type::String),
                CheckedLiteralExpression::Boolean(_) => Ok(Type::Boolean),
            },
            CheckedExpression::Variable(variable) => Ok(variable.type_.clone()),
            CheckedExpression::FunctionCall { name, arguments } => {
                match self.get_function_definition_by_name(name.name()) {
                    Some(function_definition) => Ok(function_definition.return_type),
                    None => Err(TypecheckerError::new(
                        TypecheckerErrorKind::FunctionNotDefined {
                            name: name.name().to_string(),
                        },
                        name.token().range.clone(),
                    )),
                }
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

    fn get_variable_by_name(&self, name: &str) -> Option<CheckedVariable> {
        for scope in self.scope_stack.iter().rev() {
            for variable in scope.variables.iter() {
                if variable.name == name {
                    return Some(variable.clone());
                }
            }
        }
        None
    }

    fn register_function(&mut self, function: CheckedFunctionDefinition) {
        self.functions.insert(function.name.clone(), function);
    }

    fn get_function_definition_by_name(&self, name: &str) -> Option<CheckedFunctionDefinition> {
        self.functions.get(name).cloned()
    }
}
