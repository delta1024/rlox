pub use crate::error::ScannerError as ErrorToken;
use std::{marker::PhantomData, result, slice};
pub type Result<T> = result::Result<T, ErrorToken>;
pub struct Scanner<'a> {
    tail: *const u8,
    start: *const u8,
    current: *const u8,
    line: u32,
    finished: bool,
    _marker: PhantomData<&'a str>,
}

impl Iterator for Scanner<'_> {
    type Item = Result<Token>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            self.finished = true;
            return Some(Ok(self.new_token(TokenType::EOF)));
        }
        let c = unsafe { self.advance() };
        if is_alpha(c) {
            return Some(Ok(self.identifier()));
        }
        if c.is_ascii_digit() {
            return Some(Ok(self.number()));
        }
        let id = match c {
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            ',' => TokenType::Comma,
            '.' => TokenType::Dot,
            '-' => TokenType::Minus,
            '+' => TokenType::Plus,
            ';' => TokenType::Semicolon,
            '/' => TokenType::Slash,
            '*' => TokenType::Star,
            '!' if self.matches('=') => TokenType::BangEqual,
            '!' => TokenType::Bang,
            '=' if self.matches('=') => TokenType::EqualEqual,
            '=' => TokenType::Equal,
            '>' if self.matches('=') => TokenType::GreaterEqual,
            '>' => TokenType::Greater,
            '<' if self.matches('=') => TokenType::LessEqual,
            '<' => TokenType::Less,
            '"' => return Some(self.string()),
            _ => {
                return Some(Err(ErrorToken::new("Unexpected character.", self.line)));
            }
        };
        Some(Ok(self.new_token(id)))
    }
}

fn is_alpha(c: char) -> bool {
    c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c == '_'
}

impl<'a, 'b: 'a> Scanner<'a> {
    pub fn new(source: &'b str) -> Scanner<'a> {
        Scanner {
            tail: unsafe { source.as_ptr().add(source.len()) },
            start: source.as_ptr(),
            current: source.as_ptr(),
            line: 1,
            finished: false,
            _marker: PhantomData,
        }
    }
    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            false
        } else if unsafe { self.current.read() as char == expected } {
            unsafe {
                self.advance();
            }
            true
        } else {
            false
        }
    }
    fn peek(&self) -> char {
        unsafe { self.current.read() as char }
    }

    fn peek_next(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            unsafe { self.current.add(1).read() as char }
        }
    }
    fn new_token(&self, id: TokenType) -> Token {
        Token {
            id,
            start: self.start,
            length: unsafe { self.current.offset_from(self.start) },
            line: self.line,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current == self.tail
    }

    unsafe fn advance(&mut self) -> char {
        self.current = self.current.add(1);
        self.current.sub(1).read() as char
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            match c {
                ' ' | '\r' | '\t' => unsafe {
                    self.advance();
                },
                '\n' => {
                    self.line += 1;
                    unsafe {
                        self.advance();
                    }
                }
                '/' if self.peek_next() == '/' => {
                    while self.peek() != '\n' && !self.is_at_end() {
                        unsafe {
                            self.advance();
                        }
                    }
                }
                _ => return,
            }
        }
    }

    fn check_keyword(&self, start: isize, length: isize, rest: &str, id: TokenType) -> TokenType {
        unsafe {
            if self.current.offset_from(self.start) == start + length && {
                let start = self.start.offset(start);
                let v = slice::from_raw_parts(start, length as usize);
                let actual = std::str::from_utf8(v).unwrap();
                actual == rest
            } {
                id
            } else {
                TokenType::Identifier
            }
        }
    }

    fn id_type(&mut self) -> TokenType {
        let (start, length, rest, id) = match unsafe { self.start.read() as char } {
            'a' => (1, 2, "nd", TokenType::And),
            'c' => (1, 4, "lass", TokenType::Class),
            'e' => (1, 3, "lse", TokenType::Else),
            'f' if unsafe { self.current.offset_from(self.start) } > 1 => {
                match unsafe { self.start.add(1).read() as char } {
                    'a' => (2, 3, "lse", TokenType::False),
                    'o' => (2, 1, "r", TokenType::For),
                    'u' => (2, 1, "n", TokenType::Fun),
                    _ => return TokenType::Identifier,
                }
            }
            'i' => (1, 1, "f", TokenType::If),
            'n' => (1, 2, "il", TokenType::Nil),
            'o' => (1, 1, "r", TokenType::Or),
            'p' => (1, 4, "rint", TokenType::Print),
            'r' => (1, 5, "eturn", TokenType::Return),
            's' => (1, 4, "uper", TokenType::Super),
            't' if unsafe { self.current.offset_from(self.start) } > 1 => {
                match unsafe { self.start.add(1).read() as char } {
                    'h' => (2, 2, "is", TokenType::This),
                    'r' => (2, 2, "ue", TokenType::True),
                    _ => return TokenType::Identifier,
                }
            }
            'v' => (1, 2, "ar", TokenType::Var),
            'w' => (1, 4, "hile", TokenType::While),
            _ => return TokenType::Identifier,
        };
        self.check_keyword(start, length, rest, id)
    }

    fn identifier(&mut self) -> Token {
        while is_alpha(self.peek()) || self.peek().is_ascii_digit() {
            unsafe {
                self.advance();
            }
        }

        let id = self.id_type();
        self.new_token(id)
    }

    fn string(&mut self) -> Result<Token> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            unsafe {
                self.advance();
            }
        }

        if self.is_at_end() {
            Err(ErrorToken::new("Unterminated string.", self.line))
        } else {
            unsafe {
                // The closing quote.
                self.advance();
            }
            Ok(self.new_token(TokenType::String))
        }
    }

    fn number(&mut self) -> Token {
        while self.peek().is_ascii_digit() {
            unsafe {
                self.advance();
            }
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // Consume the ".".
            unsafe {
                self.advance();
            }

            while self.peek().is_ascii_digit() {
                unsafe {
                    self.advance();
                }
            }
        }
        self.new_token(TokenType::Number)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub id: TokenType,
    pub start: *const u8,
    pub length: isize,
    pub line: u32,
}

impl Token {
    pub fn extract(&self) -> &str {
        let sli = unsafe { slice::from_raw_parts(self.start, self.length as usize) };
        let str_lis = std::str::from_utf8(sli);
        str_lis.unwrap()
    }
    pub fn null() -> Token {
        Token {
            id: TokenType::EOF,
            start: std::ptr::null(),
            length: 0,
            line: 0,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    // Single character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    Identifier,
    String,
    Number,
    // Keywords.
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Error,
    EOF,
}
