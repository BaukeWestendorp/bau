use crate::error::BauResult;
use crate::tokenizer::token::Span;
use crate::typechecker::CheckedFunctionItem;
use crate::typechecker_error;
use std::fmt::{Display, Formatter};

// FIXME: This should be an enum
#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    name: String,
    methods: Vec<CheckedFunctionItem>,
}

impl Type {
    pub fn new(name: &str, methods: Vec<CheckedFunctionItem>) -> Self {
        Self {
            name: name.to_string(),
            methods,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn methods_mut(&mut self) -> &mut Vec<CheckedFunctionItem> {
        &mut self.methods
    }

    pub fn add_method(&mut self, method: CheckedFunctionItem) -> BauResult<()> {
        if self.methods.iter().any(|m| m.name() == method.name()) {
            return typechecker_error!(
                // FIXME: Get span from method call
                Span { start: 0, end: 0 },
                "Method `{}` already exists on type `{}`",
                method.name(),
                self.name()
            );
        }
        self.methods.push(method);
        Ok(())
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
