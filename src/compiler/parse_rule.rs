use crate::lexer::TokenType;

use super::{binary, grouping, number, unary, CompilerError, Parser, Precedence};

pub(super) type ParseFn = fn(&mut Parser) -> Result<(), CompilerError>;
#[derive(Default)]
pub(super) struct ParseRule {
    pub(super) prefix: Option<ParseFn>,
    pub(super) infix: Option<ParseFn>,
    pub(super) precedence: Precedence,
}

pub(super) trait GetRule {
    type Fn;
    fn get_rule(&self) -> Option<ParseRule>;
}

impl GetRule for TokenType {
    type Fn = ParseFn;
    fn get_rule(&self) -> Option<ParseRule> {
        match self {
            Self::LeftParen => Some(ParseRule {
                prefix: Some(grouping),
                ..Default::default()
            }),
            Self::Minus => Some(ParseRule {
                prefix: Some(unary),
                infix: Some(binary),
                precedence: Precedence::Term,
            }),
            Self::Plus => Some(ParseRule {
                infix: Some(binary),
                precedence: Precedence::Term,
                ..Default::default()
            }),
            Self::Slash | Self::Star => Some(ParseRule {
                infix: Some(binary),
                precedence: Precedence::Factor,
                ..Default::default()
            }),
            Self::Number => Some(ParseRule {
                prefix: Some(number),
                ..Default::default()
            }),
            _ => None,
        }
    }
}
