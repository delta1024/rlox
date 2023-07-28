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
        use BinaryOperation as Bo;
        let (Some(b), Some(a)) = (self.pop(), self.pop()) else {
	  return Err(VmError::StackEmptyOnPopOperation);
	};
        let (Value::Int(b), Value::Int(a)) = (b, a) else {
	    return Err(VmError::WrongType);
	};
        Ok(match op {
            Bo::Add => Value::Int(a + b),
            Bo::Sub => Value::Int(a - b),
            Bo::Div => Value::Int(a / b),
            Bo::Mul => Value::Int(a * b),
        })
    }
    pub(crate) fn unary_operation(&mut self, op: UnaryOperation) -> VmResult<Value> {
        use UnaryOperation as Uo;
        let val = self.pop().ok_or(VmError::WrongType)?;
        let Value::Int(val) = val else {
	    return Err(VmError::WrongType);
	};
        Ok(match op {
            Uo::Neg => Value::Int(-val),
        })
    }
}
pub type VmResult<T> = std::result::Result<T, VmError>;
#[derive(Debug, Clone, Copy)]
pub enum VmError {
    WrongType,
    StackEmptyOnPopOperation,
}
impl fmt::Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for VmError {}
