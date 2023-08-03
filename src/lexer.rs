use std::{
    iter::Peekable,
    ops::{ControlFlow, RangeInclusive},
    str::CharIndices,
};
#[derive(Debug)]
pub struct ErrorToken {
    pub message: String,
    pub line: usize,
}

impl ErrorToken {
    pub fn new(message: impl ToString, line: usize) -> Self {
        Self {
            message: message.to_string(),
            line,
        }
    }
}
pub(crate) type LexerResult<'a> = Result<Token<'a>, ErrorToken>;
#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TokenType{
    LeftParen,  RightParen,
    LeftBrace,  RightBrace,
    Comma, Dot, Minus,  Plus,
    Semicolon, Slash,  Star,
    // One or two character tokens.
    Bang,  BangEqual,
    Equal,  EqualEqual,
    Greater,  GreaterEqual,
    Less,  LessEqual,
    // Literals.
    Identifier,  String,  Number,
    // Keywords.
    And,  Class,  Else,  False,
    For,  Fun,  If,  Nil,  Or,
    Print,  Return,  Super,  This,
    True,  Var,  While,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Token<'a> {
    pub(crate) id: TokenType,
    pub(crate) lexum: &'a str,
    pub(crate) line: usize,
}

impl<'a> Token<'a> {
    pub(crate) fn new(id: TokenType, lexum: &'a str, line: usize) -> Self {
        Self { id, lexum, line }
    }
}

pub(crate) struct Lexer<'a> {
    source: &'a str,
    start_pos: usize,
    line: usize,
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            source,
            start_pos: 0,
            line: 1,
            chars: source.char_indices().peekable(),
        }
    }
    #[inline(always)]
    fn at_end(&mut self) -> bool {
        self.chars.peek().is_none()
    }
    #[inline(always)]
    fn get_range(&self, top: usize) -> RangeInclusive<usize> {
        RangeInclusive::new(self.start_pos, top)
    }
}
impl<'a> Iterator for Lexer<'a>
where
    Self: 'a,
{
    type Item = LexerResult<'a>;
    fn next(&mut self) -> Option<Self::Item> {
	let mut line = self.line;
        let ControlFlow::Break(((cur_pos, ch), line)) = self.chars.try_for_each(move |x| {
            if x.1 == ' ' || x.1 == '\r' || x.1 == '\t' {
                ControlFlow::Continue(())
            } else if x.1 == '\n' {
		line += 1;
		ControlFlow::Continue(())
	    } else {
                ControlFlow::Break((x,line))
            }
        }) else {
	    return None;
	};
	self.line = line;
        self.start_pos = cur_pos;
        let token = match ch {
            '(' =>   Token::new(
                    TokenType::LeftParen,
                    &self.source[self.get_range(cur_pos)],
                    self.line,
                ),
            _ => todo!(),
        };
        Some(Ok(token))
    }
}
