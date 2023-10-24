use crate::interpreter::value::Value;
use crate::parser::ast::BlockKind;

#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    Continue,
    Break,
    Return(Option<Value>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub control_flow: Option<ControlFlow>,
    pub block_kind: BlockKind,
}
