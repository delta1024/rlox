use std::{collections::VecDeque, iter::Peekable, ops::ControlFlow};

use crate::{
    byte_code::OpCode,
    lexer::{Lexer, LexerResult, Token, TokenType},
};

use super::{expression, CompilerError};

pub(crate) struct Parser<'a> {
    previous: Option<Token<'a>>,
    current: Option<Token<'a>>,
    lexer: Peekable<Lexer<'a>>,
    que: VecDeque<Result<(OpCode, usize), CompilerError>>,
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<(OpCode, usize), CompilerError>;
    fn next(&mut self) -> Option<Self::Item> {
        todo!("implement this")
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
        self.lexer.peek().is_none()
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
    pub(crate) fn advance_if<T: FnOnce(&Token<'a>) -> bool>(
        &mut self,
        func: T,
    ) -> Result<Option<Token<'a>>, CompilerError> {
        if self.is_current(func) {
            self.advance()
        } else {
            Ok(None)
        }
    }
    pub(crate) fn advance_if_id(
        &mut self,
        id: TokenType,
        message: impl ToString,
    ) -> Result<Option<Token<'a>>, CompilerError> {
        if self.is_current(|t| t.id == id) {
            self.advance()
        } else {
            Err(CompilerError::new(
                self.is_at_end(),
                self.previous.unwrap_or_default(),
                message,
            ))
        }
    }
    pub(crate) fn advance_if_at_end(
        &mut self,
        message: impl ToString,
    ) -> Result<(), CompilerError> {
        if self.is_at_end() {
            Ok(())
        } else {
            Err(CompilerError::new(false, self.current.unwrap(), message))
        }
    }
}
