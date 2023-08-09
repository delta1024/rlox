use std::{fmt::Display, ops::ControlFlow};

use crate::{
    byte_code::OpCode,
    run_time::{self, runtime_error, RuntimeError, RuntimeState},
    stack::Stack,
    value::Value,
};
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

pub type VmResult<T> = std::result::Result<T, RuntimeError>;

pub(crate) struct Vm {
    pub(crate) stack: Stack<Value>,
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
    pub(crate) fn binary_instruction<'a, 'b>(
        state: &mut RuntimeState<'a, 'b>,
        instruction: BinaryOp,
    ) -> VmResult<Value> {
        let num = match instruction {
            BinaryOp::Add(Value::Number(a), Value::Number(b)) => a + b,
            BinaryOp::Sub(Value::Number(a), Value::Number(b)) => a - b,
            BinaryOp::Mul(Value::Number(a), Value::Number(b)) => a * b,
            BinaryOp::Div(Value::Number(a), Value::Number(b)) => a / b,
            _ => return runtime_error(state, "Operands must be two numbers."),
        };
        Ok(Value::Number(num))
    }
    pub(crate) fn unary_instruction<'a, 'b>(
        state: &mut RuntimeState<'a,'b> ,
        instruction: UnaryOp,
    ) -> VmResult<Value> {
        Ok(match instruction {
            UnaryOp::Negate(Value::Number(a)) => Value::Number(-a),
            _ => panic!("This is not a number"),
        })
    }
}
