use crate::{compiler::CompilerError, run_time::RuntimeError};
use std::{error::Error as StdError, fmt::Display, io::Error as IoError};

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    Compiler(CompilerError),
    Runtime(RuntimeError),
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Compiler(e) => write!(f, "{e}"),
            Self::Runtime(e) => write!(f, "{e}"),
        }
    }
}
impl From<RuntimeError> for Error {
    fn from(value: RuntimeError) -> Self {
        Self::Runtime(value)
    }
}

impl From<CompilerError> for Error {
    fn from(value: CompilerError) -> Self {
        Self::Compiler(value)
    }
}
impl From<IoError> for Error {
    fn from(value: IoError) -> Self {
        Self::Io(value)
    }
}
impl StdError for Error {}
