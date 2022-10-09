pub use crate::error::CompilerError as Error;
use crate::{
    chunk::{Chunk, OpCode},
    scanner::{Scanner, Token, TokenType},
    value::Value,
};
use std::{ptr, result};
pub type Result<T> = result::Result<T, Error>;
use rule::{get_rule, Precedence};

struct Parser<'a, 'b> {
    previous: Token,
    current: Token,
    had_error: bool,
    chunk: &'a mut Chunk,
    scanner: &'b mut Scanner<'b>,
}

impl<'a, 'b> Parser<'a, 'b> {
    fn new(scanner: &'b mut Scanner<'b>, chunk: &'a mut Chunk) -> Parser<'a, 'b> {
        let null = Token {
            id: TokenType::Error,
            start: ptr::null(),
            length: 0,
            line: 0,
        };
        Parser {
            previous: null,
            current: null,
            scanner,
            had_error: false,
            chunk,
        }
    }

    fn advance(&mut self) {
        self.previous = self.current;
        self.current = match self.scanner.next() {
            Some(token) => token,
            None => return,
        };
        if self.current.id != TokenType::Error {
            return;
        }

        let n = self.current.extract().to_owned();
        self.error_at_current(&n);
    }

    fn error_at_current(&mut self, message: &str) {
        if self.had_error {
            return;
        }
        eprint!("[line {}] Error", self.current.line);

        if self.current.id == TokenType::EOF {
            eprint!(" at end");
        } else if self.current.id == TokenType::Error {
            // do nothing
        } else {
            eprint!(" at '{}'", self.current.extract());
        }

        eprintln!(": {}", message);
        self.had_error = true;
    }

    fn error(&mut self, message: &str) {
        if self.had_error {
            return;
        }
        eprint!("[line {}] Error", self.previous.line);

        if self.previous.id == TokenType::EOF {
            eprint!(" at end");
        } else if self.previous.id == TokenType::Error {
            // do nothing
        } else {
            eprint!(" at '{}'", self.previous.extract());
        }

        eprintln!(": {}", message);
        self.had_error = true;
    }

    fn consume(&mut self, id: TokenType, message: &str) {
        if self.current.id == id {
            self.advance();
            return;
        }

        self.error_at_current(message)
    }

    fn emit_byte<T: Into<u8>>(&mut self, byte: T) {
        let line = self.previous.line;
        self.chunk.write(byte, line);
    }
    fn emit_bytes<T: Into<U>, U: Into<u8>>(&mut self, byte1: T, byte2: U) {
        self.emit_byte(byte1.into());
        self.emit_byte(byte2);
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return);
    }

    fn emit_constant<T: Into<Value>>(&mut self, value: T) {
        let n = self.chunk.constant(value.into());
        self.emit_bytes(OpCode::Constant, n);
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        #[cfg(feature = "print_code")]
        if !self.had_error {
            self.chunk.set_name("code");
            println!("{:?}", self.chunk);
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = match get_rule(self.previous.id).prefix {
            Some(rule) => rule,
            None => {
                self.error("Expected expression.");
                return;
            }
        };

        prefix_rule(self);

        while precedence <= get_rule(self.current.id).precedence {
            self.advance();
            let infix_rule = get_rule(self.previous.id).infix.unwrap();
            infix_rule(self);
        }
    }
}

fn number(parser: &mut Parser) {
    let value = parser.previous.extract().parse::<f64>().unwrap();
    parser.emit_constant(value);
}

fn grouping(parser: &mut Parser) {
    expression(parser);
    parser.consume(TokenType::RightParen, "Expect ')' after expression");
}
fn unary(parser: &mut Parser) {
    let op_type = parser.previous.id;

    // Compile the operand.
    parser.parse_precedence(Precedence::Uanry);

    match op_type {
        TokenType::Minus => parser.emit_byte(OpCode::Negate),
        _ => unreachable!(),
    }
}
fn binary(parser: &mut Parser) {
    let op_type = parser.previous.id;
    let rule = rule::get_rule(op_type);
    parser.parse_precedence(rule.precedence.add_one());

    parser.emit_byte(match op_type {
        TokenType::Plus => OpCode::Add,
        TokenType::Minus => OpCode::Subtract,
        TokenType::Star => OpCode::Multiply,
        TokenType::Slash => OpCode::Divide,
        _ => unreachable!(),
    });
}
fn expression(parser: &mut Parser) {
    parser.parse_precedence(Precedence::Assignment);
}

pub fn compile(source: &str) -> Result<Chunk> {
    let mut chunk = Chunk::new();
    let mut scanner = Scanner::new(source);
    let mut parser = Parser::new(&mut scanner, &mut chunk);
    parser.advance();
    expression(&mut parser);
    parser.consume(TokenType::EOF, "Expect end of expression.");
    parser.end_compiler();
    if !parser.had_error {
        Ok(chunk)
    } else {
        Err(Error)
    }
}

mod rule {
    #[rustfmt::skip]
    const RULES: [ParseRule; 40] = [
        // TokenType::LeftParen
        ParseRule{ prefix: Some(grouping), infix: None         , precedence: Precedence::None  },
        // TokenType::RightParen
        ParseRule{ prefix: None         , infix: None         , precedence: Precedence::None  },
        // TokenType::LeftBrace
        ParseRule{ prefix: None         , infix: None         , precedence: Precedence::None  },
        // TokenType::RightBrace
        ParseRule{ prefix: None         , infix: None         , precedence: Precedence::None  },
        // TokenType::Comma
        ParseRule{ prefix: None         , infix: None         , precedence: Precedence::None  },
        // TokenType::Dot
        ParseRule{ prefix: None         , infix: None         , precedence: Precedence::None  },
        // TokenType::Minus
        ParseRule{ prefix: Some(unary)  , infix: Some(binary) , precedence: Precedence::Term  },
        // TokenType::Plus
        ParseRule{ prefix: None         , infix: Some(binary) , precedence: Precedence::Term  },
        // TokenType::Semicolon
        ParseRule{ prefix: None         , infix: None         , precedence: Precedence::None  },
        // TokenType::Slash
        ParseRule{ prefix: None         , infix: Some(binary) , precedence: Precedence::Factor},
        // TokenType::Star
        ParseRule{ prefix: None         , infix: Some(binary) , precedence: Precedence::Factor},
        // TokenType::Bang
        ParseRule{ prefix: None         , infix: None         , precedence: Precedence::None  },
        // TokenType::BangEqual
        ParseRule{ prefix: None         , infix: None         , precedence: Precedence::None  },
        // TokenType::Equal
        ParseRule{ prefix: None         , infix: None         , precedence: Precedence::None  },
        // TokenType::EqualEqual
        ParseRule{ prefix: None         , infix: None         , precedence: Precedence::None  },
        // TokenType::Greater
        ParseRule{ prefix: None         , infix: None         , precedence: Precedence::None  },
        // TokenType::GreaterEqual
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::Less        
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::LessEqual   
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::Identifier  
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::String      
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::Number      
        ParseRule{ prefix:  Some(number) , infix: None        , precedence: Precedence::None  },
        // TokenType::And         
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::Class       
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::Else        
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::False       
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::For         
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::Fun         
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::If          
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::Nil         
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::Or          
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::Print       
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::Return      
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::Super       
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::This        
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::True        
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::Var         
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::While       
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::Error        
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
        // TokenType::EOF         
        ParseRule{ prefix:  None        , infix: None         , precedence: Precedence::None  },
    ];
    use super::{binary, grouping, number, unary, Parser};
    use crate::scanner::TokenType;
    pub(super) type ParseFn = fn(&mut Parser);
    #[derive(Clone, Copy)]
    pub(super) struct ParseRule {
        pub(super) prefix: Option<ParseFn>,
        pub(super) infix: Option<ParseFn>,
        pub(super) precedence: Precedence,
    }

    #[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
    pub(super) enum Precedence {
        None,
        Assignment,
        Or,
        And,
        Equality,
        Comparison,
        Term,
        Factor,
        Uanry,
        Call,
        Primary,
        Overflow,
    }

    impl Precedence {
        pub(super) fn add_one(&self) -> Self {
            match *self {
                Self::None => Self::Assignment,
                Self::Assignment => Self::Or,
                Self::Or => Self::And,
                Self::And => Self::Equality,
                Self::Equality => Self::Comparison,
                Self::Comparison => Self::Term,
                Self::Term => Self::Factor,
                Self::Factor => Self::Uanry,
                Self::Uanry => Self::Call,
                Self::Call => Self::Primary,
                Self::Primary => Self::Overflow,
                Self::Overflow => unreachable!(),
            }
        }
    }

    pub(super) fn get_rule(id: TokenType) -> ParseRule {
        RULES[id as usize]
    }
}
