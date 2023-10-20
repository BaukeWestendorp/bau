use crate::error::BauResult;
use crate::interpreter::value::Value;
use crate::interpreter::Interpreter;
use crate::parser::ast::{BlockKind, Stmt};
use crate::parser::ast::{Expr, Item};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref BUILTIN_FUNCTIONS: Vec<BuiltinFunction> = vec![BuiltinFunction {
        function: Item::Function {
            name: "print".to_string(),
            parameters: vec![],
            body: Stmt::Block {
                statements: vec![],
                block_kind: BlockKind::Function
            },
        },
        action: builtin_print,
    },];
}

pub fn from_name(name: &str) -> Option<BuiltinFunction> {
    for builtin in BUILTIN_FUNCTIONS.iter() {
        if builtin.name() == name {
            return Some(builtin.clone());
        }
    }
    None
}

fn builtin_print(interpreter: &mut Interpreter, args: &Vec<Expr>) -> BauResult<Value> {
    let value = interpreter.execute_expression(&args[0])?;
    println!("{}", value);
    Ok(Value::Option(None))
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuiltinFunction {
    pub function: Item,
    action: fn(&mut Interpreter, &Vec<Expr>) -> BauResult<Value>,
}

impl BuiltinFunction {
    pub fn name(&self) -> String {
        match &self.function {
            Item::Function { name, .. } => name.clone(),
        }
    }

    pub fn call(&self, interpreter: &mut Interpreter, args: &Vec<Expr>) -> BauResult<Value> {
        (self.action)(interpreter, args)
    }
}
