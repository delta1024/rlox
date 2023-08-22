pub mod error;
pub(crate) type CompilerResult<T> = Result<T, CompilerError>;
pub(crate) mod parser;
mod functions;
mod parse_rule;
mod precedence;

pub use self::error::*;
pub(crate) use parser::*;
use functions::*;
use precedence::*;




