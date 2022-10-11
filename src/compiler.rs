pub use crate::error::CompilerError as Error;
use crate::{
    chunk::{Chunk, OpCode},
    objects::allocate_string,
    scanner::{self, Scanner, Token, TokenType},
    value::Value,
};
use std::{ptr, result};
pub type Result<T> = result::Result<T, Error>;
use rule::{get_rule, Precedence};

struct Parser<'a, 'b> {
    previous: Token,
    current: Token,
    had_error: Result<()>,
    chunk: &'a mut Chunk,
    scanner: &'b mut Scanner<'b>,
}

impl<'a, 'b> Parser<'a, 'b> {
    fn new(scanner: &'b mut Scanner<'b>, chunk: &'a mut Chunk) -> Parser<'a, 'b> {
        let null = scanner::Token {
            id: TokenType::Error,
            start: ptr::null(),
            length: 0,
            line: 0,
        };
        Parser {
            previous: null,
            current: null,
            scanner,
            had_error: Ok(()),
            chunk,
        }
    }

    fn advance(&mut self) -> Result<()> {
        self.previous = self.current;
        self.current = match self.scanner.next() {
            Some(token) => match token {
                Ok(token) => token,
                Err(err) => return self.error_at_current(err.extract()),
            },
            None => return Ok(()),
        };
        Ok(())
    }
    fn error_at(parser: &mut Parser, token: Token, message: &str) -> Result<()> {
        let mut error = format!("[line {}] Error", token.line);
        if token.id == TokenType::EOF {
            error.push_str(" at end");
        } else {
            error.push_str(" at '");
            error.push_str(token.extract());
            error.push('\'');
        }
        error.push_str(": ");
        error.push_str(message);
        if let Err(_) = parser.had_error {
            Err(Error(error))
        } else {
            parser.had_error = Err(Error(error));
            parser.had_error.clone()
        }
    }

    fn error_at_current(&mut self, message: &str) -> Result<()> {
        Self::error_at(self, self.current, message)
    }

    fn error(&mut self, message: &str) -> Result<()> {
        Self::error_at(self, self.previous, message)
    }

    fn syncronize(&mut self) {
        while self.current.id != TokenType::EOF {
            if self.previous.id == TokenType::Semicolon {
                return;
            }
            match self.current.id {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {
                    if let Err(_) = self.advance() {
                        // Do nothing.
                    }
                }
            }
        }
    }

    fn consume(&mut self, id: TokenType, message: &str) -> Result<()> {
        if self.current.id == id {
            self.advance()
        } else {
            self.error_at_current(message)
        }
    }

    fn check(&self, id: TokenType) -> bool {
        self.current.id == id
    }

    fn matches(&mut self, id: TokenType) -> Result<bool> {
        if !self.check(id) {
            return Ok(false);
        }

        self.advance()?;
        Ok(true)
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
        if self.had_error.is_ok() {
            self.chunk.set_name("code");
            println!("{:?}", self.chunk);
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<()> {
        self.advance()?;
        let prefix_rule = match get_rule(self.previous.id).prefix {
            Some(rule) => rule,
            None => {
                return self.error("Expected expression.");
            }
        };
        let can_assign = precedence <= Precedence::Assignment;
        prefix_rule(self, can_assign)?;

        while precedence <= get_rule(self.current.id).precedence {
            self.advance()?;
            let infix_rule = get_rule(self.previous.id).infix.unwrap();
            infix_rule(self, can_assign)?;
        }
        if can_assign && self.matches(TokenType::Equal)? {
            return self.error("Invalid assignment target.");
        }
        Ok(())
    }

    fn identifier_constant(&mut self, name: Token) -> u8 {
        let string = allocate_string(name.extract());
        self.chunk.constant(string.into())
    }

    fn parse_variable(&mut self, error_message: &str) -> Result<u8> {
        self.consume(TokenType::Identifier, error_message)?;
        Ok(self.identifier_constant(self.previous))
    }

    fn define_variable(&mut self, global: u8) {
        self.emit_bytes(OpCode::DefineGlobal, global);
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) -> Result<()> {
        let arg = self.identifier_constant(name);
        let n;
        if can_assign && self.matches(TokenType::Equal).unwrap() {
            expression(self)?;
            n = OpCode::SetGlobal;
        } else {
            n = OpCode::GetGlobal;
        }
        self.emit_bytes(n, arg);
        Ok(())
    }
}

fn number(parser: &mut Parser, _: bool) -> Result<()> {
    let value = parser.previous.extract().parse::<f64>().unwrap();
    parser.emit_constant(value);
    Ok(())
}

fn string(parser: &mut Parser, _: bool) -> Result<()> {
    let string = allocate_string(parser.previous.extract());
    parser.emit_constant(string);
    Ok(())
}

fn variable(parser: &mut Parser, can_assign: bool) -> Result<()> {
    parser.named_variable(parser.previous, can_assign)
}

fn grouping(parser: &mut Parser, _: bool) -> Result<()> {
    match expression(parser) {
        _ => parser.consume(TokenType::RightParen, "Expect ')' after expression"),
    }
}

fn unary(parser: &mut Parser, _: bool) -> Result<()> {
    let op_type = parser.previous.id;

    // Compile the operand.
    parser.parse_precedence(Precedence::Uanry)?;

    parser.emit_byte(match op_type {
        TokenType::Minus => OpCode::Negate,
        TokenType::Bang => OpCode::Not,
        _ => unreachable!(),
    });
    Ok(())
}

fn binary(parser: &mut Parser, _: bool) -> Result<()> {
    let op_type = parser.previous.id;
    let rule = rule::get_rule(op_type);
    parser.parse_precedence(rule.precedence.add_one())?;

    parser.emit_byte(match op_type {
        TokenType::BangEqual => {
            parser.emit_bytes(OpCode::Equal, OpCode::Not);
            return Ok(());
        }
        TokenType::EqualEqual => OpCode::Equal,
        TokenType::Greater => OpCode::Greater,
        TokenType::GreaterEqual => {
            parser.emit_bytes(OpCode::Less, OpCode::Not);
            return Ok(());
        }
        TokenType::Less => OpCode::Less,
        TokenType::LessEqual => {
            parser.emit_bytes(OpCode::Greater, OpCode::Not);
            return Ok(());
        }
        TokenType::Plus => OpCode::Add,
        TokenType::Minus => OpCode::Subtract,
        TokenType::Star => OpCode::Multiply,
        TokenType::Slash => OpCode::Divide,
        _ => unreachable!(),
    });
    Ok(())
}

fn literal(parser: &mut Parser, _: bool) -> Result<()> {
    parser.emit_byte(match parser.previous.id {
        TokenType::False => OpCode::False,
        TokenType::Nil => OpCode::Nil,
        TokenType::True => OpCode::True,
        _ => unreachable!(),
    });
    Ok(())
}

fn expression(parser: &mut Parser) -> Result<()> {
    parser.parse_precedence(Precedence::Assignment)
}

fn var_declaration(parser: &mut Parser) -> Result<()> {
    let global = parser.parse_variable("Expect variable name.")?;

    if parser.matches(TokenType::Equal)? {
        expression(parser)?;
    } else {
        parser.emit_byte(OpCode::Nil);
    }
    parser.consume(
        TokenType::Semicolon,
        "Expect ';' after variable declaration.",
    )?;
    parser.define_variable(global);

    Ok(())
}

fn expression_statement(parser: &mut Parser) -> Result<()> {
    expression(parser)?;
    parser.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
    parser.emit_byte(OpCode::Pop);
    Ok(())
}

fn print_statement(parser: &mut Parser) -> Result<()> {
    expression(parser)?;
    parser.consume(TokenType::Semicolon, "Expect ';' after value.")?;
    parser.emit_byte(OpCode::Print);
    Ok(())
}

fn declaration(parser: &mut Parser) {
    let n;
    if let Ok(true) = parser.matches(TokenType::Var) {
        n = var_declaration(parser);
    } else {
        n = statement(parser);
    }
    if let Err(err) = n {
        eprintln!("{}", err);
        parser.syncronize();
    }
}

fn statement(parser: &mut Parser) -> Result<()> {
    if parser.matches(TokenType::Print)? {
        print_statement(parser)
    } else {
        expression_statement(parser)
    }
}

pub fn compile(source: &str) -> Result<Chunk> {
    let mut chunk = Chunk::new();
    let mut scanner = Scanner::new(source);
    let mut parser = Parser::new(&mut scanner, &mut chunk);
    parser.advance()?;
    while !parser.matches(TokenType::EOF)? {
        declaration(&mut parser);
    }
    parser.end_compiler();
    if let Err(err) = parser.had_error {
        Err(err)
    } else {
        Ok(chunk)
    }
}

mod rule {
    #[rustfmt::skip]
    const RULES: [ParseRule; 40] = [
        // TokenType::LeftParen
        ParseRule{ prefix: Some(grouping) , infix: None         , precedence: Precedence::None  },
        // TokenType::RightParen
        ParseRule{ prefix: None           , infix: None         , precedence: Precedence::None  },
        // TokenType::LeftBrace
        ParseRule{ prefix: None           , infix: None         , precedence: Precedence::None  },
        // TokenType::RightBrace
        ParseRule{ prefix: None           , infix: None         , precedence: Precedence::None  },
        // TokenType::Comma
        ParseRule{ prefix: None           , infix: None         , precedence: Precedence::None  },
        // TokenType::Dot
        ParseRule{ prefix: None           , infix: None         , precedence: Precedence::None  },
        // TokenType::Minus
        ParseRule{ prefix: Some(unary)    , infix: Some(binary) , precedence: Precedence::Term  },
        // TokenType::Plus
        ParseRule{ prefix: None           , infix: Some(binary) , precedence: Precedence::Term  },
        // TokenType::Semicolon
        ParseRule{ prefix: None           , infix: None         , precedence: Precedence::None  },
        // TokenType::Slash
        ParseRule{ prefix: None           , infix: Some(binary) , precedence: Precedence::Factor},
        // TokenType::Star
        ParseRule{ prefix: None           , infix: Some(binary) , precedence: Precedence::Factor},
        // TokenType::Bang
        ParseRule{ prefix: Some(unary)    , infix: None         , precedence: Precedence::None  },
        // TokenType::BangEqual
        ParseRule{ prefix: None           , infix: Some(binary) , precedence: Precedence::Equality  },
        // TokenType::Equal
        ParseRule{ prefix: None           , infix: None         , precedence: Precedence::None  },
        // TokenType::EqualEqual
        ParseRule{ prefix: None           , infix: Some(binary) , precedence: Precedence::Equality  },
        // TokenType::Greater
        ParseRule{ prefix: None           , infix: Some(binary) , precedence: Precedence::Comparison  },
        // TokenType::GreaterEqual
        ParseRule{ prefix:  None          , infix: Some(binary) , precedence: Precedence::Comparison  },
        // TokenType::Less        
        ParseRule{ prefix:  None          , infix: Some(binary) , precedence: Precedence::Comparison  },
        // TokenType::LessEqual   
        ParseRule{ prefix:  None          , infix: Some(binary) , precedence: Precedence::Comparison  },
        // TokenType::Identifier  
        ParseRule{ prefix:  Some(variable), infix: None         , precedence: Precedence::None  },
        // TokenType::String      
        ParseRule{ prefix:  Some(string)  , infix: None         , precedence: Precedence::None  },
        // TokenType::Number      
        ParseRule{ prefix:  Some(number)  , infix: None         , precedence: Precedence::None  },
        // TokenType::And         
        ParseRule{ prefix:  None          , infix: None         , precedence: Precedence::None  },
        // TokenType::Class       
        ParseRule{ prefix:  None          , infix: None         , precedence: Precedence::None  },
        // TokenType::Else        
        ParseRule{ prefix:  None          , infix: None         , precedence: Precedence::None  },
        // TokenType::False       
        ParseRule{ prefix:  Some(literal) , infix: None         , precedence: Precedence::None  },
        // TokenType::For         
        ParseRule{ prefix:  None          , infix: None         , precedence: Precedence::None  },
        // TokenType::Fun         
        ParseRule{ prefix:  None          , infix: None         , precedence: Precedence::None  },
        // TokenType::If          
        ParseRule{ prefix:  None          , infix: None         , precedence: Precedence::None  },
        // TokenType::Nil         
        ParseRule{ prefix:  Some(literal) , infix: None         , precedence: Precedence::None  },
        // TokenType::Or          
        ParseRule{ prefix:  None          , infix: None         , precedence: Precedence::None  },
        // TokenType::Print       
        ParseRule{ prefix:  None          , infix: None         , precedence: Precedence::None  },
        // TokenType::Return      
        ParseRule{ prefix:  None          , infix: None         , precedence: Precedence::None  },
        // TokenType::Super       
        ParseRule{ prefix:  None          , infix: None         , precedence: Precedence::None  },
        // TokenType::This        
        ParseRule{ prefix:  None          , infix: None         , precedence: Precedence::None  },
        // TokenType::True        
        ParseRule{ prefix:  Some(literal) , infix: None         , precedence: Precedence::None  },
        // TokenType::Var         
        ParseRule{ prefix:  None          , infix: None         , precedence: Precedence::None  },
        // TokenType::While       
        ParseRule{ prefix:  None          , infix: None         , precedence: Precedence::None  },
        // TokenType::Error        
        ParseRule{ prefix:  None          , infix: None         , precedence: Precedence::None  },
        // TokenType::EOF         
        ParseRule{ prefix:  None          , infix: None         , precedence: Precedence::None  },
    ];
    use super::{binary, grouping, literal, number, string, unary, variable, Parser};
    use crate::scanner::TokenType;
    pub(super) type ParseFn = fn(&mut Parser, bool) -> super::Result<()>;
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
