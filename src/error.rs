use std::{error::Error, fmt};

#[derive(Debug, PartialEq)]
pub enum VmError {
    Compile,
    Runtime,
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Error for VmError {}
#[derive(Debug)]
pub struct CompilerError;
impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error during compilation")
    }
}
impl Error for CompilerError {}
impl From<CompilerError> for VmError {
    fn from(_: CompilerError) -> Self {
        VmError::Compile
    }
}
