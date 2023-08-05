use std::{collections::VecDeque, iter::Peekable, ops::ControlFlow};

use crate::{
    byte_code::OpCode,
    lexer::{Lexer, LexerResult, Token, TokenType},
};

use super::{expression, CompilerError};

pub(crate) struct Parser<'a> {
    pub(super) previous: Token<'a>,
    pub(super) lexer: Peekable<Lexer<'a>>,
    pub(super) que: VecDeque<Result<(OpCode, usize), CompilerError>>,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(lexer: Lexer<'a>) -> Self {
        Self {
            previous: Token::default(),
            lexer: lexer.peekable(),
            que: VecDeque::new(),
        }
    }
    pub(super) fn advance(&mut self) -> Result<(), CompilerError> {
        match self.lexer.next() {
            Some(Ok(token)) => {
                self.previous = token;
            }
            Some(Err(err)) => return Err(err.into()),
            None => self.previous = Token::default(),
        }
        Ok(())
    }
    pub(super) fn advance_if(
        &mut self,
        pred: TokenType,
        message: impl ToString,
    ) -> Result<(), CompilerError> {
        match self.lexer.peek() {
            Some(Ok(t)) => {
                if t.id == pred {
                    return self.advance();
                }
                Err(self.error_at_current(message))
            }
            None => {
                if pred == TokenType::None {
                    return Ok(());
                }
                Err(self.error_at_current(message))
            }
            _ => Ok(()),
        }
    }
    pub(super) fn peek_current(&mut self) -> Option<&Token<'a>> {
        match self.lexer.peek() {
            Some(Ok(t)) => Some(t),
            _ => None,
        }
    }
    pub(super) fn error_at_current(&mut self, message: impl ToString) -> CompilerError {
        match self.lexer.peek() {
            Some(Err(err)) => err.clone().into(),
            Some(Ok(err)) => CompilerError::new(false, *err, message),
            None => CompilerError::new(true, Token::default(), message),
        }
    }
    pub(super) fn error(&mut self, message: impl ToString) -> CompilerError {
        CompilerError::new(self.is_at_end(), self.previous, message)
    }
    fn is_at_end(&mut self) -> bool {
        self.lexer.peek().is_none() && self.previous.id == TokenType::None
    }
    pub(super) fn emit_byte(&mut self, op_code: OpCode) {
        let line = self.previous.line;
        self.que.push_back(Ok((op_code, line)));
    }
    pub(super) fn end_compiler(&mut self) {
        self.emit_return();
    }
    pub(super) fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return);
    }
}

impl<'a> Iterator for Parser<'a>
where
    Self: 'a,
{
    type Item = Result<(OpCode, usize), CompilerError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.que.is_empty() && self.lexer.peek().is_some() {
            match expression(self) {
                Ok(()) => (),
                Err(err) => return Some(Err(err)),
            }
            if let Err(err) = self.advance_if(TokenType::None, "Expected end of expression") {
                return Some(Err(err));
            }
            self.end_compiler();
        }
        self.que.pop_front()
    }
}
