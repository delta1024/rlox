use std::{error::Error, fmt};

#[derive(Debug, PartialEq)]
pub enum VmError {
    Compile(String),
    Runtime(String),
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Compile(err) | Self::Runtime(err) => err,
            }
        )
    }
}
impl Error for VmError {}
#[derive(Debug, Clone)]
pub struct ParserError(pub String);
impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for ParserError {}
impl From<ParserError> for VmError {
    fn from(err: ParserError) -> Self {
        VmError::Compile(err.0)
    }
}

#[derive(Debug, Clone)]
pub struct CompilerError(pub String);
impl CompilerError {
    pub fn new<T>(message: &str) -> Result<T, Self> {
        Err(Self(String::from(message)))
    }
}
impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for CompilerError {}
impl From<CompilerError> for ParserError {
    fn from(s: CompilerError) -> Self {
        Self(s.0)
    }
}
impl From<ParserError> for CompilerError {
    fn from(e: ParserError) -> Self {
        Self(e.0)
    }
}
impl From<CompilerError> for String {
    fn from(s: CompilerError) -> Self {
        s.0
    }
}
#[derive(Debug, Clone, Copy)]
pub struct ScannerError {
    pub start: *const u8,
    pub length: isize,
    pub line: u32,
}
impl ScannerError {
    pub fn new(message: &str, line: u32) -> ScannerError {
        ScannerError {
            start: message.as_ptr(),
            length: message.len() as isize,
            line,
        }
    }

    pub fn extract(&self) -> &str {
        let sli = unsafe { std::slice::from_raw_parts(self.start, self.length as usize) };
        let str_lis = std::str::from_utf8(sli);
        str_lis.unwrap()
    }
}
impl From<ScannerError> for ParserError {
    fn from(s: ScannerError) -> Self {
        Self(String::from(s.extract()))
    }
}
impl fmt::Display for ScannerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.extract())
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
