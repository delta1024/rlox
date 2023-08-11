pub mod error;
mod parse_rule;
pub(crate) mod parser;
mod precedence;

pub use error::*;
use parse_rule::*;
pub(crate) use parser::*;
use precedence::*;

use crate::{byte_code::OpCode, lexer::TokenType, value::Value};

fn parse_precedence<'a>(parser: &mut Parser<'a>, prec: Precedence) -> Result<(), CompilerError> {
todo!()
}
fn expression<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
todo!()
}
fn number<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    todo!()
}
fn grouping<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    todo!()
}
fn unary<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    todo!()
}
fn literal<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    todo!()
}
fn binary<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    todo!()
}
