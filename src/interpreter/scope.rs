use crate::interpreter::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    Continue,
    Break,
    Return(Option<Value>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub control_flow: Option<ControlFlow>,
}
