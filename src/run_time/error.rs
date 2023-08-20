use std::fmt::Display;

use super::{vm::VmResult, RuntimeState};

#[derive(Clone, Default, Debug)]
pub struct RuntimeError {
    message: String,
    line: u8,
}
#[macro_export]
macro_rules! runtime_error {
    ($runtime:expr, $($args:tt)*) => {
	crate::run_time::error::runtime_error($runtime, std::format_args!($($args)*))
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error: {}\n[line {}] in script\n",
            self.message, self.line
        )
    }
}
impl std::error::Error for RuntimeError {}

pub(crate) fn runtime_error<'a, 'b, T>(
    state: &mut RuntimeState<'a, 'b>,
    message: impl ToString,
) -> VmResult<T> {
    let pos = state.get_position();
    let line = state.get_frames().chunk.get_line(pos).unwrap();
    state.get_vm().stack.reset();
    Err(RuntimeError {
        message: message.to_string(),
        line,
    })
}
