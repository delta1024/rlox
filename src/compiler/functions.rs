use crate::{error as comp_error, cur_matches,byte_code::OpCode, lexer::{Token, TokenType}};
use super::{parse_rule::*, Parser, Precedence, CompilerResult, CompilerError};
macro_rules! sync {
    ($parser:expr, $err: expr) => {
        $parser.que.push_back(Err($err));
        $parser.syncronize()
    };
}

pub(super) fn parse_precedence<'a>(parser: &mut Parser<'a>, prec: Precedence) -> CompilerResult<()> {
    parser.advance()?;
    let Some(parse_rule) = parser.map_previous(|t| t.id.get_rule().map(|r| r.prefix).flatten()).flatten() else {
	comp_error!(parser, "Expect expression.");
    };
    let can_assign = prec <= Precedence::Assignment;
    parse_rule(parser, can_assign)?;

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
        infix_rule(parser, can_assign)?;
    }
    if can_assign && cur_matches!(parser, Equal) {
        comp_error!(parser, "Invalid assignment target.");
    }
    Ok(())
}
pub(super) fn var_declaration<'a>(parser: &mut Parser<'a>) -> CompilerResult<()> {
    let global = parser.parse_variable("Expect variable name.")?;
    if cur_matches!(parser, Equal) {
        expression(parser)?;
    } else {
        parser.emit_byte(OpCode::Nil);
    }
    parser.advance_if_id(
        TokenType::Semicolon,
        "Expect ';' after variable declaration.",
    )?;
    parser.define_variable(global);
    Ok(())
}
pub(super) fn decleration<'a>(parser: &mut Parser<'a>) {
    if match parser.matches(Some(TokenType::Var)) {
        Ok(b) => b,
        Err(err) => {
            sync!(parser, err);
            false
        }
    } {
        if let Err(err) = var_declaration(parser) {
            sync!(parser, err);
        }
    } else if let Err(err) = statement(parser) {
        sync!(parser, err);
    }
}
pub(super) fn named_variable<'a>(
    parser: &mut Parser<'a>,
    token: Token<'a>,
    can_assign: bool,
) -> CompilerResult<()> {
    let arg = parser.identifier_constant(token);
    if can_assign && cur_matches!(parser, Equal) {
        expression(parser)?;
        parser.emit_byte(OpCode::SetGlobal(arg));
    } else {
        parser.emit_byte(OpCode::GetGlobal(arg));
    }
    Ok(())
}
pub(super) fn variable<'a>(parser: &mut Parser<'a>, can_assign: bool) -> CompilerResult<()> {
    let token = parser.map_previous(|t| *t).unwrap();
    named_variable(parser, token, can_assign)
}
pub(super) fn print_statement<'a>(parser: &mut Parser<'a>) -> CompilerResult<()> {
    expression(parser)?;
    parser.advance_if_id(TokenType::Semicolon, "Expect ';' after value.")?;
    parser.emit_byte(OpCode::Print);
    Ok(())
}
pub(super) fn expression_statement<'a>(parser: &mut Parser<'a>) -> CompilerResult<()> {
    expression(parser)?;
    parser.advance_if_id(TokenType::Semicolon, "Expect ';' after expression.")?;
    parser.emit_byte(OpCode::Pop);
    Ok(())
}
pub(super) fn statement<'a>(parser: &mut Parser<'a>) -> CompilerResult<()> {
    if cur_matches!(parser, Print) {
        print_statement(parser)?;
    } else {
        expression_statement(parser)?;
    }
    Ok(())
}
pub(super) fn expression<'a>(parser: &mut Parser<'a>) -> CompilerResult<()> {
    parse_precedence(parser, Precedence::Assignment)
}
pub(super) fn number<'a>(parser: &mut Parser<'a>, _: bool) -> CompilerResult<()> {
    let num = parser
        .map_previous(|t| t.lexum)
        .unwrap()
        .parse::<i64>()
        .unwrap();
    parser.emit_byte(OpCode::Constant(num.into()));
    Ok(())
}
pub(super) fn grouping<'a>(parser: &mut Parser<'a>, _: bool) -> CompilerResult<()> {
    expression(parser)?;
    parser
        .advance_if_id(TokenType::RightParen, "Expect ')' after expression.")
        .map(|_| ())
}

pub(super) fn string<'a>(parser: &mut Parser<'a>, _: bool) -> CompilerResult<()> {
    let s = parser.map_previous(|t| t.lexum).unwrap();
    let o = parser.allocator.allocate_string(s);
    parser.emit_byte(OpCode::Constant(o.into()));
    Ok(())
}
pub(super) fn unary<'a>(parser: &mut Parser<'a>, _: bool) -> CompilerResult<()> {
    let id = parser.map_previous(|t| t.id).unwrap();
    parse_precedence(parser, Precedence::Unary)?;
    match id {
        TokenType::Minus => parser.emit_byte(OpCode::Neg),
        TokenType::Bang => parser.emit_byte(OpCode::Not),
        _ => unreachable!(),
    }
    Ok(())
}
pub(super) fn literal<'a>(parser: &mut Parser<'a>, _: bool) -> CompilerResult<()> {
    match parser.map_previous(|t| t.id).unwrap() {
        TokenType::Nil => parser.emit_byte(OpCode::Nil),
        TokenType::True => parser.emit_byte(OpCode::True),
        TokenType::False => parser.emit_byte(OpCode::False),
        _ => unreachable!(),
    }
    Ok(())
}
pub(super) fn binary<'a>(parser: &mut Parser<'a>, _: bool) -> CompilerResult<()> {
    let op_type = parser.map_previous(|t| t.id).unwrap();
    let rule = op_type.get_rule().unwrap();
    parse_precedence(parser, rule.precedence + 1)?;
    match op_type {
        TokenType::Plus => parser.emit_byte(OpCode::Add),
        TokenType::Minus => parser.emit_byte(OpCode::Sub),
        TokenType::Star => parser.emit_byte(OpCode::Mul),
        TokenType::Slash => parser.emit_byte(OpCode::Div),
        TokenType::BangEqual => parser.emit_bytes(OpCode::Equal, OpCode::Not),
        TokenType::EqualEqual => parser.emit_byte(OpCode::Equal),
        TokenType::Greater => parser.emit_byte(OpCode::Greater),
        TokenType::GreaterEqual => parser.emit_bytes(OpCode::Less, OpCode::Not),
        TokenType::Less => parser.emit_byte(OpCode::Less),
        TokenType::LessEqual => parser.emit_bytes(OpCode::Greater, OpCode::Not),

        _ => unreachable!(),
    }
    Ok(())
}
