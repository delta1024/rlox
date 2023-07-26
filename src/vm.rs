use crate::{
    stack::{StackResult, ValueStack},
    value::Value,
};
use std::fmt;
pub(crate) enum BinaryOperation {
    Add,
    Sub,
    Div,
    Mul,
}
pub(crate) enum UnaryOperation {
    Neg,
}
pub(crate) struct Vm {
    stack: ValueStack,
}

impl Vm {
    pub(crate) fn new() -> Self {
        Self {
            stack: ValueStack::new(),
        }
    }
    pub(crate) fn push(&mut self, value: Value) -> StackResult {
        self.stack.push(value)
    }
    pub(crate) fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }
    pub(crate) fn binary_operation(&mut self, op: BinaryOperation) -> VmResult<Value> {
        use BinaryOperation as BO;
        match op {
            BO::Add => todo!(),
            BO::Sub => todo!(),
            BO::Div => todo!(),
            BO::Mul => todo!(),
        }
    }
    pub(crate) fn unary_operation(&mut self, op: UnaryOperation) -> VmResult<Value> {
        use UnaryOperation as UO;
        match op {
            UO::Neg => todo!(),
        }
    }
}
pub type VmResult<T> = std::result::Result<T, VmError>;
#[derive(Debug, Clone, Copy)]
pub enum VmError {
    StackEmptyOnPopOperation,
}
impl fmt::Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for VmError {}
