use std::{error::Error, fmt};

#[derive(Debug, PartialEq)]
pub enum VmError {
    Compile,
    Runtime(String),
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Compile => "",
                Self::Runtime(err) => err,
            }
        )
    }
}
impl Error for VmError {}
#[derive(Debug)]
pub struct CompilerError;
impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}
impl Error for CompilerError {}
impl From<CompilerError> for VmError {
    fn from(error: CompilerError) -> Self {
        VmError::Compile
    }
}

#[derive(Debug)]
pub struct ValueError;

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Wrong Value type.")
    }
}
impl Error for ValueError {}
