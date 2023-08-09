pub mod error;
mod parse_rule;
pub(crate) mod parser;
mod precedence;

pub use error::*;
use parse_rule::*;
pub(crate) use parser::*;
use precedence::*;

use crate::{byte_code::OpCode, lexer::TokenType};

fn parse_precedence<'a>(parser: &mut Parser<'a>, prec: Precedence) -> Result<(), CompilerError> {
    parser.advance()?;
    let Some(prefix_rule) = parser.previous.id.get_rule().map(|x| x.prefix).flatten() else {
	return Err(parser.error("Expect expression."));
    };
    prefix_rule(parser)?;

    while Some(prec)
        <= parser
            .peek_current()
            .map(|x| x.id.get_rule().map(|x| x.precedence))
            .flatten()
    {
        parser.advance()?;

        if let Some(infix_rule) = parser.previous.id.get_rule().map(|x| x.infix).flatten() {
            infix_rule(parser)?;
        }
    }

    Ok(())
}
fn expression<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    parse_precedence(parser, Precedence::Assignment)
}
fn number<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    let num = parser.previous.lexum.parse::<i64>().unwrap();
    parser.emit_byte(OpCode::Constant(num));
    Ok(())
}
fn grouping<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    expression(parser)?;
    parser.advance_if(TokenType::RightParen, "Expect ')' after expression.")
}
fn unary<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    let op_type = parser.previous.id;

    parse_precedence(parser, Precedence::Unary)?;

    match op_type {
        TokenType::Minus => parser.emit_byte(OpCode::Neg),
        _ => unreachable!(),
    }
    Ok(())
}
fn binary<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    let op_type = parser.previous.id;
    let rule = op_type.get_rule().unwrap().precedence;
    parse_precedence(parser, rule + 1)?;
    match op_type {
        TokenType::Plus => parser.emit_byte(OpCode::Add),
        TokenType::Minus => parser.emit_byte(OpCode::Sub),
        TokenType::Star => parser.emit_byte(OpCode::Mul),
        TokenType::Slash => parser.emit_byte(OpCode::Div),
        _ => unreachable!(),
    }
    Ok(())
}
