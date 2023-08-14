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
    parser.advance()?;
    let Some(parse_rule) = parser.map_previous(|t| t.id.get_rule().map(|r| r.prefix).flatten()).flatten() else {

	return Err(CompilerError::new(parser.map_previous(|t| *t), "Expect expression.", parser.map_previous(|t| t.line).unwrap_or_default())); 
    };
    parse_rule(parser)?;

    while Some(true)
        == parser
            .map_current(|t| t.id.get_rule().map(|r| prec <= r.precedence))
            .flatten()
    {
        parser.advance()?;
        let infix_rule = parser
            .map_previous(|t| t.id.get_rule().map(|r| r.infix).flatten())
            .flatten()
            .unwrap();
	infix_rule(parser)?;
    }
    Ok(())
}
fn expression<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    parse_precedence(parser, Precedence::Assignment)
}
fn number<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    let num = parser
        .map_previous(|t| t.lexum)
        .unwrap()
        .parse::<i64>()
        .unwrap();
    parser.emit_byte(OpCode::Constant(num.into()));
    Ok(())
}
fn grouping<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    expression(parser)?;
    parser
        .advance_if_id(TokenType::RightParen, "Expect ')' after expression.")
        .map(|_| ())
}
fn unary<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    let id = parser.map_previous(|t| t.id).unwrap();
    parse_precedence(parser, Precedence::Unary)?;
    match id {
        TokenType::Minus => parser.emit_byte(OpCode::Neg),
        _ => unreachable!(),
    }
    Ok(())
}
fn literal<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    match parser.map_previous(|t| t.id).unwrap() {
        TokenType::Nil => parser.emit_byte(OpCode::Nil),
        TokenType::True => parser.emit_byte(OpCode::True),
        TokenType::False => parser.emit_byte(OpCode::False),
        _ => unreachable!(),
    }
    Ok(())
}
fn binary<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    let op_type = parser.map_previous(|t| t.id).unwrap();
    let rule = op_type.get_rule().unwrap();
    parse_precedence(parser, rule.precedence + 1)?;
    match op_type {
        TokenType::Plus => parser.emit_byte(OpCode::Add),
        TokenType::Minus => parser.emit_byte(OpCode::Sub),
        TokenType::Star => parser.emit_byte(OpCode::Mul),
        TokenType::Slash => parser.emit_byte(OpCode::Div),
        _ => unreachable!(),
    }
    Ok(())
}
