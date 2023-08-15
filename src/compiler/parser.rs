use std::{collections::VecDeque, iter::Peekable};

use crate::{
    byte_code::OpCode,
    error_at_current,
    lexer::{Lexer, Token, TokenType},
};

use super::{expression, CompilerError};
#[derive(Debug)]
pub(crate) struct Parser<'a> {
    previous: Option<Token<'a>>,
    current: Option<Token<'a>>,
    lexer: Peekable<Lexer<'a>>,
    que: VecDeque<(OpCode, usize)>,
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<(OpCode, usize), CompilerError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.previous.is_none() && self.current.is_none() {
            if let Err(err) = self.advance() {
                return Some(Err(err));
            }
        }
        if self.que.is_empty() && !self.is_at_end() {
            if let Err(err) = expression(self) {
                return Some(Err(err));
            }
            if let Err(err) = self.advance_if_at_end("Expect end of expression.") {
                return Some(Err(err));
            }
            self.end_compiler();
        }
        self.que.pop_front().map(|t| Ok(t))
    }
}

impl<'a> Default for Parser<'a> {
    fn default() -> Self {
        Self {
            previous: None,
            current: None,
            lexer: Lexer::new("").peekable(),
            que: VecDeque::new(),
        }
    }
}

impl<'a> Parser<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            lexer: Lexer::new(source).peekable(),
            ..Default::default()
        }
    }
    pub(crate) fn emit_byte(&mut self, op_code: OpCode) {
        let line = self.map_previous(|t| t.line).unwrap();
        self.que.push_back((op_code, line));
    }
    pub(crate) fn emit_bytes(&mut self, op_code: OpCode, op_code2: OpCode) {
	self.emit_byte(op_code);
	self.emit_byte(op_code2);
    }
    pub(crate) fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return);
    }
    pub(crate) fn end_compiler(&mut self) {
        self.emit_return();
    }
    pub(crate) fn map_previous<T: FnOnce(&Token<'a>) -> U, U>(&self, func: T) -> Option<U> {
        self.previous.as_ref().map(func)
    }
    pub(crate) fn map_current<T: FnOnce(&Token<'a>) -> U, U>(&self, func: T) -> Option<U> {
        self.current.as_ref().map(func)
    }
    pub(crate) fn is_previous<T: FnOnce(&Token<'a>) -> bool>(&self, func: T) -> bool {
        self.previous.as_ref().map_or(false, func)
    }
    pub(crate) fn is_current<T: FnOnce(&Token<'a>) -> bool>(&self, func: T) -> bool {
        self.current.as_ref().map_or(false, func)
    }
    pub(crate) fn is_at_end(&mut self) -> bool {
        self.current.is_none()
    }
    pub(crate) fn advance(&mut self) -> Result<Option<Token<'a>>, CompilerError> {
        self.previous = self.current;
        let x = match self.lexer.next() {
            Some(Ok(t)) => Some(t),
            Some(Err(err)) => return Err(err.into()),
            None => None,
        };
        self.current = x;
        Ok(self.current)
    }
    // pub(crate) fn advance_if<T: FnOnce(&Token<'a>) -> bool>(
    //     &mut self,
    //     func: T,
    // ) -> Result<Option<Token<'a>>, CompilerError> {
    //     if self.is_current(func) {
    //         self.advance()
    //     } else {
    //         Ok(None)
    //     }
    // }
    pub(crate) fn advance_if_id(
        &mut self,
        id: TokenType,
        message: impl ToString,
    ) -> Result<Option<Token<'a>>, CompilerError> {
        if self.is_current(|t| t.id == id) {
            self.advance()
        } else {
            error_at_current!(self, "{}", message.to_string())
        }
    }
    pub(crate) fn advance_if_at_end(
        &mut self,
        message: impl ToString,
    ) -> Result<(), CompilerError> {
        if self.is_at_end() {
            Ok(())
        } else {
            Err(CompilerError::new(
                None,
                message,
                self.current.map(|t| t.line).unwrap_or_default(),
            ))
        }
    }
}
