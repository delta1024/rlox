use std::{collections::VecDeque, iter::Peekable};

use crate::{
    byte_code::OpCode,
    error_at_current,
    heap::{Allocator, ObjPtr, ObjString},
    lexer::{Lexer, Token, TokenType},
};

use super::{decleration, CompilerError, CompilerResult};
#[derive(Debug)]
pub(crate) struct Parser<'a> {
    previous: Option<Token<'a>>,
    current: Option<Token<'a>>,
    lexer: Peekable<Lexer<'a>>,
    pub(super) que: VecDeque<CompilerResult<(OpCode, usize)>>,
    pub(super) allocator: Allocator,
    line: usize,
}

impl<'a> Iterator for Parser<'a> {
    type Item = CompilerResult<(OpCode, usize)>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.previous.is_none() && self.current.is_none() {
            if let Err(err) = self.advance() {
                self.que.push_back(Err(err));
            }
        }
        if self.que.is_empty() && !self.is_at_end() {
            while !match self.matches(None) {
                Ok(b) => b,
                Err(err) => {
                    self.que.push_back(Err(err));
                    // The only error we're going to encounter here is
                    // an unterminated string therefore we must be at
                    // the end so we report true and exit the loop.
                    true
                }
            } {
                decleration(self);
            }
            self.end_compiler();
        }
        self.que.pop_front()
    }
}

impl<'a> Default for Parser<'a> {
    fn default() -> Self {
        Self {
            previous: None,
            current: None,
            lexer: Lexer::new("").peekable(),
            que: VecDeque::new(),
            allocator: Allocator::new(std::ptr::null_mut()),
            line: 0,
        }
    }
}

impl<'a> Parser<'a> {
    pub(crate) fn new(source: &'a str, allocator: Allocator) -> Self {
        Self {
            lexer: Lexer::new(source).peekable(),
            allocator,
            ..Default::default()
        }
    }
    pub(crate) fn emit_byte(&mut self, op_code: OpCode) {
        self.que.push_back(Ok((op_code, self.line)));
    }
    pub(crate) fn emit_bytes(&mut self, op_code: OpCode, op_code2: OpCode) {
        self.emit_byte(op_code);
        self.emit_byte(op_code2);
    }
    pub(crate) fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return);
    }
    pub(crate) fn define_variable(&mut self, name: ObjPtr<ObjString>) {
	self.emit_byte(OpCode::DefineGlobal(name));
    }
    fn identifier_constant(&mut self, name: Token<'a>) -> ObjPtr<ObjString> {
	self.allocator.allocate_string(name.lexum).as_obj()
    }
    pub(crate) fn parse_variable(&mut self, err_message: impl ToString) -> CompilerResult<ObjPtr<ObjString>> {
	self.advance_if_id(TokenType::Identifier, err_message)?;
	Ok(self.identifier_constant(self.previous.unwrap()))
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
    pub(crate) fn is_current<T: FnOnce(&Token<'a>) -> bool>(&self, func: T) -> bool {
        self.current.as_ref().map_or(false, func)
    }
    pub(crate) fn is_at_end(&mut self) -> bool {
        self.current.is_none()
    }
    pub(crate) fn advance(&mut self) -> CompilerResult<Option<Token<'a>>> {
        self.previous = self.current;
        let x = match self.lexer.next() {
            Some(Ok(t)) => {
                self.line = t.line;
                Some(t)
            }
            Some(Err(err)) => return Err(err.into()),
            None => None,
        };
        self.current = x;
        Ok(self.current)
    }
    pub(crate) fn advance_if_id(
        &mut self,
        id: TokenType,
        message: impl ToString,
    ) -> CompilerResult<Option<Token<'a>>> {
        if self.is_current(|t| t.id == id) {
            self.advance()
        } else {
            error_at_current!(self, "{}", message.to_string())
        }
    }
    pub(crate) fn check(&self, id: Option<TokenType>) -> bool {
        self.current.as_ref().map(|t| t.id) == id
    }
    pub(crate) fn matches(&mut self, id: Option<TokenType>) -> Result<bool, CompilerError> {
        if !self.check(id) {
            return Ok(false);
        }
        self.advance()?;
        Ok(true)
    }
    pub(crate) fn syncronize(&mut self) {
        while self.current.is_some() {
            if self.map_previous(|t| t.id) == Some(TokenType::Semicolon) {
                return;
            }

            match self.map_current(|t| t.id).unwrap() {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {
                    if let Err(err) = self.advance() {
                        self.que.push_back(Err(err));
                    }
                }
            }
        }
    }
}
#[macro_export]
macro_rules! cur_matches {
    ($parser:expr, $token:tt) => {
        match $parser.matches(Some(TokenType::$token)) {
            Ok(b) => b,
            Err(err) => return Err(err),
        }
    };
}
