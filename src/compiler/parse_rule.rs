use std::ops::ControlFlow;

use crate::lexer::TokenType;

use super::{binary, grouping, number, unary, CompilerError, Parser, Precedence};

pub(super) type ParseFn = fn(&mut Parser) -> Result<(), CompilerError>;

pub(super) struct ParseRule {
    pub(super) prefix: Option<ParseFn>,
    pub(super) infix: Option<ParseFn>,
    pub(super) precedence: Precedence,
}

impl ParseRule {
    pub(super) fn new(prefix: ParseFn, infix: ParseFn, precedence: Precedence) -> Self {
        Self {
            prefix: Some(prefix),
            infix: Some(infix),
            precedence,
        }
    }
}

impl<'a> Default for ParseRule {
    fn default() -> Self {
        Self {
            prefix: None,
            infix: None,
            precedence: Precedence::default(),
        }
    }
}

pub(super) trait GetRule {
    fn get_rule<'a>(&self) -> Option<ParseRule>;
}

impl GetRule for TokenType {
    fn get_rule<'a>(&self) -> Option<ParseRule> {
        match self {
            Self::LeftParen => Some(ParseRule {
                prefix: Some(grouping),
                ..Default::default()
            }),
            Self::Minus => Some(ParseRule::new(unary, binary, Precedence::Term)),
            Self::Plus => Some(ParseRule {
                infix: Some(binary),
                precedence: Precedence::Term,
                ..Default::default()
            }),
            Self::Star | Self::Slash => Some(ParseRule {
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
