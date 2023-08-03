mod parse_rule;
mod parser;
mod precedence;

use std::fmt::Display;

pub use parse_rule::*;
pub use parser::*;
pub use precedence::*;

use crate::{
    byte_code::{Chunk, OpCode, CompilationResult},
    lexer::{Lexer, TokenType},
};
#[derive(Debug, Clone)]
pub enum CompilerError {
    NotRule,
    Scanner,
    NumberParse,
    Consume(String),
}

impl std::error::Error for CompilerError {}

impl Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub(crate) fn compile(source: &str) -> CompilationResult {
    let lexer = Lexer::new(source).peekable();
    let parser = Parser::new(lexer);
    //    parser.advance();
    // expression(&mut parser)?;
    // parser
    //     .advance_if_eq(TokenType::Eof)
    //     .map_err(|_| CompilerError::Scanner)?;
    // parser.end_compiler();

    // Ok(parser.chunk.into())
    parser.collect()
}
fn parse_precedence<'a>(
    parser: &mut Parser<'a>,
    precedence: Precedence,
) -> Result<(), CompilerError> {
    parser.advance();
    let Some(prefix_rule) = parser.previous.as_ref().unwrap().id.get_rule().unwrap().prefix else {
	return Err(CompilerError::NotRule);
    };
    prefix_rule(parser)?;
    while let Some(t) = parser
        .lexer
        .next_if(|t| Some(precedence) <= t.id.get_rule().map(|p| p.precedence))
    {
        parser.previous = Some(t.clone());
        if let Some(infix_rule) = t.id.get_rule().map(|r| r.infix).unwrap() {
            infix_rule(parser)?;
        }
    }
    Ok(())
}
fn expression<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    parse_precedence(parser, Precedence::Assignment)
}
fn number<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    let value: i64 = parser
        .previous
        .clone()
        .ok_or(CompilerError::NumberParse)?
        .lexum
        .parse()
        .map_err(|_| CompilerError::NumberParse)?;
    parser.emit_byte(OpCode::Constant(value.into()));
    Ok(())
}
fn grouping<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    expression(parser)?;
    parser
        .advance_if_eq(TokenType::RightParen)
        .map_err(|_| CompilerError::Consume("Expect ')' after expression".to_string()))
}

fn unary<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    let op_type = parser.previous.as_ref().map(|t| t.id).unwrap();
    parse_precedence(parser, Precedence::Unary)?;
    match op_type {
        TokenType::Minus => parser.emit_byte(OpCode::Neg),
        _ => unreachable!(),
    }
    Ok(())
}

fn binary<'a>(parser: &mut Parser<'a>) -> Result<(), CompilerError> {
    let op_type = parser.previous.as_ref().map(|t| t.id).unwrap();
    let rule = op_type.get_rule().unwrap();
    parse_precedence(parser, rule.precedence + 1)?;
    use TokenType::*;
    match op_type {
        Plus => parser.emit_byte(OpCode::Add),
        Minus => parser.emit_byte(OpCode::Sub),
        Star => parser.emit_byte(OpCode::Mul),
        Slash => parser.emit_byte(OpCode::Div),
        _ => unreachable!(),
    }
    Ok(())
}
