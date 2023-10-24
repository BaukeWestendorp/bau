use crate::error::BauResult;
use crate::interpreter::value::Value;
use crate::interpreter::Interpreter;
use crate::parser::ast::BlockKind;
use crate::typechecker::{CheckedExpr, CheckedFunctionItem, CheckedStmt, VOID_TYPE_ID};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref BUILTIN_FUNCTIONS: Vec<BuiltinFunction> = vec![BuiltinFunction {
        function: CheckedFunctionItem::new(
            "print",
            VOID_TYPE_ID,
            vec![],
            CheckedStmt::Block {
                statements: vec![],
                block_kind: BlockKind::Function
            }
        ),
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

fn builtin_print(
    interpreter: &mut Interpreter,
    args: &Vec<CheckedExpr>,
) -> BauResult<Option<Value>> {
    let value = interpreter.execute_expression(&args[0])?;
    println!(
        "{}",
        value.map(|v| v.to_string()).unwrap_or("void".to_string())
    );
    Ok(None)
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuiltinFunction {
    pub function: CheckedFunctionItem,
    action: fn(&mut Interpreter, &Vec<CheckedExpr>) -> BauResult<Option<Value>>,
}

impl BuiltinFunction {
    pub fn name(&self) -> &str {
        self.function.name()
    }

    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        args: &Vec<CheckedExpr>,
    ) -> BauResult<Option<Value>> {
        (self.action)(interpreter, args)
    }
}
