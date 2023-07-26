use crate::{frame::CallFrame, value::Value};
use std::{
    fmt,
    ops::{Deref, DerefMut},
};

const STACK_MAX: usize = 256;
pub type StackResult = std::result::Result<(), StackOverlowError>;
pub(crate) struct ValueStack {
    stack_top: usize,
    values: [Value; STACK_MAX],
}
impl ValueStack {
    pub(crate) fn new() -> Self {
        Self {
            stack_top: 0,
            values: [Value::Nil; STACK_MAX],
        }
    }

    pub(crate) fn push(&mut self, val: Value) -> StackResult {
        if self.stack_top == STACK_MAX {
            return Err(StackOverlowError);
        }
        self.stack_top += 1;
        self.values[self.stack_top - 1] = val;
        Ok(())
    }

    pub(crate) fn pop(&mut self) -> Option<Value> {
        if self.stack_top == 0 {
            return None;
        }

        self.stack_top -= 1;
        Some(self.values[self.stack_top])
    }

    pub(crate) fn len(&self) -> usize {
        self.stack_top
    }
}

impl Deref for ValueStack {
    type Target = [Value];
    fn deref(&self) -> &Self::Target {
        &self.values[..]
    }
}
#[derive(Debug)]
pub struct StackOverlowError;
impl fmt::Display for StackOverlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for StackOverlowError {}
const CALL_STACK_MAX: usize = 64;
#[repr(transparent)]
pub(crate) struct CallStack(Vec<CallFrame>);
impl CallStack {
    pub(crate) fn new() -> CallStack {
        CallStack(Vec::with_capacity(CALL_STACK_MAX))
    }
    pub(crate) fn push(&mut self, frame: CallFrame) -> StackResult {
        if self.len() == CALL_STACK_MAX {
            return Err(StackOverlowError);
        }
        self.0.push(frame);
        Ok(())
    }
    pub(crate) fn pop(&mut self) -> Option<CallFrame> {
        self.0.pop()
    }
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}
impl Deref for CallStack {
    type Target = [CallFrame];
    fn deref(&self) -> &Self::Target {
        &self.0[..]
    }
}
impl DerefMut for CallStack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0[..]
    }
}
