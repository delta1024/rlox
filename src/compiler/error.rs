 use std::fmt::Display;

use crate::lexer::{ErrorToken, Token, TokenType};
#[macro_export]
macro_rules! error {
    ($parser:expr, $($arg:tt)*) => {
return	 Err(CompilerError::new(
	    $parser.map_previous(|t| *t),
	    std::format_args!($($arg)*),
	    $parser.map_previous(|t| t.line).unwrap_or_default()))
    };
}
#[macro_export]
macro_rules! error_at_current {
    ($parser:expr, $($arg:tt)*) => {
	return Err(CompilerError::new(
	    $parser.map_current(|t| *t),
	    std::format_args!($($arg)*),
	    $parser.map_current(|t| t.line).unwrap_or_default()))
    }

}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerError {
    token: Option<ErrorToken>,
    line: usize,
    message: String,
    from_lexer: bool,
}

impl std::error::Error for CompilerError {}
impl<'a> CompilerError {
    pub(crate) fn new(token: Option<Token<'a>>, message: impl ToString, line: usize) -> Self {
        Self {
            token: token.map(|t| ErrorToken::new(t.lexum.to_string(), t.line)),
            message: message.to_string(),
            line,
            from_lexer: false,
        }
    }
}

impl<'a> Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut message = self.message.clone();
        write!(f, "[line {}] Error", self.line)?;
        if self.token.is_none() {
            write!(f, " at end")?;
        } else if self.from_lexer {
            let t = self.token.clone().unwrap();
            message = t.message;
        } else if let Some(tn) = self.token.as_ref() {
            write!(f, " at '{}'", tn.message)?;
        }
        write!(f, ": {}", message)
    }
}

impl<'a> From<ErrorToken> for CompilerError {
    fn from(value: ErrorToken) -> Self {
        todo!()
    }
}

