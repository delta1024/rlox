use std::{collections::VecDeque, iter::Peekable};

use crate::{
    byte_code::OpCode,
    lexer::{Lexer, Token, TokenType},
};

use super::{expression, CompilerError};

#[derive(Debug, Clone)]
pub(super) struct Parser<'a> {
    que: VecDeque<(OpCode, usize)>,
    pub(super) previous: Option<Token<'a>>,
    pub(super) lexer: Peekable<Lexer<'a>>,
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<(OpCode, usize), CompilerError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.que.is_empty() && self.lexer.peek().is_some() {
            expression(self).unwrap();
            if let Err(err) = self
                .lexer
                .next_if(|t| t.id == TokenType::Eof)
                .ok_or(CompilerError::Consume("Expected end of file".to_string()))
            {
                return Some(Err(err));
            } // Chech for Eof
            self.end_compiler();
        }
        self.que.pop_front().map(|x| Ok(x))
    }
}
impl<'a> Parser<'a> {
    pub(super) fn new(lexer: Peekable<Lexer<'a>>) -> Self {
        Self {
            que: VecDeque::new(),
            previous: None,
            lexer,
        }
    }
    pub(super) fn advance(&mut self) {
        self.previous = self.lexer.next();
    }
    pub(super) fn advance_if_eq(&mut self, id: TokenType) -> Result<(), ()> {
        if let Some(token) = self.lexer.next_if(|t| t.id == id) {
            self.previous = Some(token);
            return Ok(());
        }
        Err(())
    }
    pub(super) fn emit_byte(&mut self, byte: OpCode) {
        self.que
            .push_back((byte, self.previous.map(|x| x.line).unwrap_or(0)));
    }
    pub(super) fn end_compiler(&mut self) {
        self.emit_byte(OpCode::Return);
    }
    pub(super) fn get_cur_type(&mut self) -> Option<TokenType> {
        self.lexer.peek().map(|t| t.id)
    }
}

impl<'a> Parser<'a> {
    fn get_current_line(&mut self) -> usize {
        self.lexer.peek().map(|x| x.line).unwrap_or(0)
    }
}
