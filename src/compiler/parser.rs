use std::iter::Peekable;

use crate::{
    byte_code::{ChunkBuilder, OpCode},
    lexer::{Lexer, Token, TokenType},
};

#[derive(Debug)]
pub(super) struct Parser<'a> {
    pub(super) chunk: ChunkBuilder,
    pub(super) previous: Option<Token<'a>>,
    pub(super) lexer: Peekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    pub(super) fn new(lexer: Peekable<Lexer<'a>>) -> Self {
        Self {
            chunk: ChunkBuilder::new(),
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
        let line = self.get_current_line() as u8;
        self.chunk.write_byte(byte, line);
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
