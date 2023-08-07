use std::fmt::Display;

use crate::lexer::{ErrorToken, Token, TokenType};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerError {
    at_end: bool,
    id: TokenType,
    lexum: String,
    line: usize,
    message: String,
}

impl std::error::Error for CompilerError {}
impl<'a> CompilerError {
    pub(crate) fn new(at_end: bool, token: Token<'a>, message: impl ToString) -> Self {
        Self {
            at_end,
            id: token.id,
            lexum: token.lexum.to_string(),
            line: token.line,
            message: message.to_string(),
        }
    }
}

impl<'a> Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] Error", self.line)?;
        if self.at_end {
            write!(f, " at end")?;
        } else {
            write!(f, " at '{}'", self.lexum)?;
        }
        write!(f, ": {}", self.message)
    }
}

impl<'a> From<ErrorToken> for CompilerError {
    fn from(value: ErrorToken) -> Self {
        Self {
            at_end: false,
            id: TokenType::None,
            line: value.line,
            lexum: String::new(),
            message: value.message,
        }
    }
}
