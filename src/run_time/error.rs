use std::{fmt::Display, ops::ControlFlow};

use super::vm::VmResult;

use super::RuntimeState;

#[derive(Clone, Default, Debug)]
pub struct RuntimeError {
    message: String,
    line: u8,
}

impl RuntimeError {
    pub fn new(message: String, line: u8) -> Self {
        Self { message, line }
    }
}
impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n[line {}] in script\n", self.message, self.line)
    }
}
impl std::error::Error for RuntimeError {}

pub(crate) fn runtime_error<'a,'b,T>(
    state: &mut RuntimeState<'a, 'b>,
    message: impl ToString,
) -> VmResult<T>{
    let pos = state.get_position();
    let line = state.get_frames().chunk.get_line(pos).unwrap();
    state.get_vm().stack.reset();
    Err(RuntimeError { message: message.to_string(), line })
}
