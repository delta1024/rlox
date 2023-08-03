use std::fmt::Display;

use crate::lexer::{ErrorToken, Token};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerError<'a> {
    at_end: bool,
    token: Result<Token<'a>, usize>,
    message: String,
}

impl<'a> CompilerError<'a> {
    pub(crate) fn new(at_end: bool, token: Token<'a>, message: impl ToString) -> Self {
        Self {
            at_end,
            token: Ok(token),
            message: message.to_string(),
        }
    }
    pub fn get_line(&self) -> usize {
        match self.token.as_ref() {
            Ok(t) => t.line,
            Err(l) => *l,
        }
    }
}

impl<'a> Display for CompilerError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] Error", self.get_line())?;
        if self.at_end {
            write!(f, " at end")?;
        } else if let Ok(t) = self.token.as_ref() {
            write!(f, " at '{}'", t.lexum)?;
        }
        write!(f, ": {}", self.message)
    }
}

impl<'a> From<ErrorToken> for CompilerError<'a> {
    fn from(value: ErrorToken) -> Self {
        Self {
            at_end: false,
            token: Err(value.line),
            message: value.message,
        }
    }
}
