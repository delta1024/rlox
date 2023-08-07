use std::fmt::Display;

use crate::{byte_code::OpCode, stack::Stack, value::Value};
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum BinaryOp {
    Add(Value, Value),
    Sub(Value, Value),
    Mul(Value, Value),
    Div(Value, Value),
}

impl BinaryOp {
    pub(crate) fn new(op_code: OpCode, a: Value, b: Value) -> Self {
        match op_code {
            OpCode::Add => Self::Add(a, b),
            OpCode::Sub => Self::Sub(a, b),
            OpCode::Mul => Self::Mul(a, b),
            OpCode::Div => Self::Div(a, b),
            _ => unreachable!(),
        }
    }
}

pub(crate) enum UnaryOp {
    Negate(Value),
}
impl UnaryOp {
    pub(crate) fn new(op: OpCode, a: Value) -> Self {
        match op {
            OpCode::Neg => Self::Negate(a),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct VmError(String);
impl Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	write!(f, "{}", self.0)
    }
}
impl std::error::Error for VmError {}

pub type VmResult<T> = std::result::Result<T, VmError>;

pub(crate) struct Vm {
    stack: Stack<Value>,
}

impl Vm {
    pub(crate) fn new() -> Self {
        Self {
            stack: Stack::new(),
        }
    }
    #[inline(always)]
    pub(crate) fn push(&mut self, value: Value) {
	self.stack.push(value);
    }
    #[inline(always)]
    pub(crate) fn pop(&mut self) -> Option<Value> {
	self.stack.pop()
    }
    pub(crate) fn binary_instruction(&mut self, instruction: BinaryOp) -> VmResult<Value> {
        Ok(match instruction {
            BinaryOp::Add(a, b) => a + b,
            BinaryOp::Sub(a, b) => a - b,
            BinaryOp::Mul(a, b) => a * b,
            BinaryOp::Div(a, b) => a / b,
        })
    }
    pub(crate) fn unary_instruction(&mut self, instruction: UnaryOp) -> VmResult<Value> {
	Ok(match instruction{
	    UnaryOp::Negate(a) => -a,
	})
    }
}
