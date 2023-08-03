use std::{iter::Peekable, collections::VecDeque};

use crate::{
    byte_code::OpCode,
    lexer::{Lexer, Token, TokenType},
};

use self::lock::PushLock;

use super::{expression, CompilerError};
mod lock {
    use std::{borrow::BorrowMut, cell::RefCell, collections::VecDeque, mem};

    use crate::{byte_code::OpCode, compiler::CompilerError};

    #[derive(Debug, Clone)]
    pub(super) enum State {
        Open(VecDeque<Result<(OpCode, usize), CompilerError>>),
        Closed(VecDeque<Result<(OpCode, usize), CompilerError>>),
    }
    #[derive(Debug, Clone)]
    pub(super) struct PushLock(pub(super) RefCell<State>);
    impl PushLock {
        pub(super) fn new() -> Self {
            Self(RefCell::new(State::Open(VecDeque::new())))
        }
        pub(super) fn lock(&mut self) {}
        pub(super) fn unlock(&mut self) {
            let n = match &mut *self.0.borrow_mut() {
                State::Open(c) | State::Closed(c) => {
                    let n = std::mem::take(c);
                    State::Closed(n)
                }
            };
            self.0.replace(n);
        }

        /// Returns `true` if the push lock is [`Open`].
        ///
        /// [`Open`]: PushLock::Open
        #[must_use]
        pub(super) fn is_open(&self) -> bool {
            matches!(&*self.0.borrow(), State::Open(..))
        }

        /// Returns `true` if the push lock is [`Closed`].
        ///
        /// [`Closed`]: PushLock::Closed
        #[must_use]
        pub(super) fn is_closed(&self) -> bool {
            matches!(&*self.0.borrow(), State::Closed(..))
        }
        pub(super) fn push(&mut self, r: Result<(OpCode, usize), CompilerError>) {
            match &mut *self.0.borrow_mut() {
                State::Open(a) | State::Closed(a) => a.push_back(r),
            }
        }
        pub(super) fn pop(&mut self) -> Option<Result<(OpCode, usize), CompilerError>> {
            match &mut *self.0.borrow_mut() {
                State::Closed(a) | State::Open(a) => a.pop_front(),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct Parser<'a> {
    que: VecDeque<Result<(OpCode, usize), CompilerError>>,
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
                self.que.push_back(Err(err));
            } // Chech for Eof
            self.end_compiler();
            
        }
        self.que.pop_front()
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
            .push_back(Ok((byte, self.previous.map(|x| x.line).unwrap_or(0))));
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
