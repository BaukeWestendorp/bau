use crate::error::{BauError, BauResult};
use crate::interpreter::Interpreter;
use crate::parser::ast::Item;

impl Interpreter {
    pub fn evaluate_top_level(&mut self, top_level: Vec<Item>) -> BauResult<()> {
        let mut last_result = Ok(());
        for item in top_level {
            last_result = match item {
                function @ Item::Function { .. } => self.evaluate_function_item(function),
            };

            if last_result.is_err() {
                return last_result;
            }
        }
        last_result
    }

    pub fn evaluate_function_item(&mut self, function: Item) -> BauResult<()> {
        match &function {
            Item::Function { name, .. } => {
                self.functions.insert(name.clone(), function);
                Ok(())
            }
        }
    }
}
