pub use crate::error::ParserError as Error;
use crate::{
    chunk::{Chunk, OpCode},
    error::CompilerError,
    memory::GarbageCollector,
    objects::{allocate_string, Obj, ObjFunction, ObjString},
    scanner::{self, Scanner, Token, TokenType},
    value::Value,
};
use std::{ptr, result};
pub type Result<T> = result::Result<T, Error>;
type CmpResult<T> = result::Result<T, CompilerError>;
use rule::{get_rule, Precedence};
const U8_MAX: usize = u8::MAX as usize;
#[derive(PartialEq, PartialOrd)]
enum FunctionType {
    Function,
    Script,
}
static mut COMPILER: *mut Compiler = std::ptr::null_mut();
struct Compiler {
    function: *mut ObjFunction,
    enclosing: *mut Compiler,
    id: FunctionType,
    locals: [Local; U8_MAX],
    upvalues: [Upvalue; U8_MAX],
    local_count: usize,
    scope_depth: isize,
}

impl Compiler {
    fn new(id: FunctionType, enclosing: *mut Compiler, name: *mut ObjString) -> Compiler {
        let m = if id != FunctionType::Script {
            ObjFunction::new(name)
        } else {
            ObjFunction::new(ptr::null_mut())
        };
        let mut n = Compiler {
            function: m,
            id,
            locals: [Local::null(); U8_MAX],
            upvalues: [Upvalue::new(); U8_MAX],
            local_count: 0,
            scope_depth: 0,
            enclosing,
        };
        let local = &mut n.locals[n.local_count];
        n.local_count += 1;
        local.depth = 0;
        local.name = Token::null();

        n
    }

    fn add_local(&mut self, name: Token) -> CmpResult<()> {
        if self.local_count == U8_MAX {
            return CompilerError::new("Too many local variables in function.");
        }
        let local = &mut self.locals[self.local_count];
        self.local_count += 1;
        local.name = name;
        local.depth = -1;
        Ok(())
    }

    fn resolve_local(&self, name: Token) -> CmpResult<Option<u8>> {
        if self.local_count > 0 {
            for i in (0..=(self.local_count - 1)).rev() {
                let local = &self.locals[i];
                if name.extract() == local.name.extract() {
                    return Ok(Some(i as u8));
                }
                if local.depth == -1 {
                    return CompilerError::new(
                        "Cannot initialize variable in its own declaration.",
                    );
                }
            }
        }
        Ok(None)
    }

    fn add_upvalue(&mut self, index: u8, is_local: bool) -> CmpResult<u8> {
        let upvalue_count = self.function().upvalue_count as usize;
        for i in 0..upvalue_count {
            let upvalue = self.upvalues[i];
            if upvalue.index as u8 == index && upvalue.is_local == is_local {
                return Ok(i as u8);
            }
        }
        if upvalue_count == U8_MAX {
            return CompilerError::new("Too many closure variables in function.");
        }
        self.upvalues[upvalue_count].is_local = is_local;
        self.upvalues[upvalue_count].index = index as isize;
        self.function().upvalue_count += 1;
        Ok(self.function().upvalue_count as u8 - 1)
    }
    fn resolve_upvalue(&mut self, name: Token) -> CmpResult<Option<u8>> {
        if let Some(enclosing) = self.enclosing() {
            if let Some(local) = enclosing.resolve_local(name)? {
                enclosing.locals[local as usize].is_captured = true;
                Ok(Some(self.add_upvalue(local, true)?))
            } else {
                if let Some(upvalue) = enclosing.resolve_upvalue(name)? {
                    Ok(Some(self.add_upvalue(upvalue, false)?))
                } else {
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }
    fn mark_initialized(&mut self) {
        if self.scope_depth == 0 {
            return;
        }
        self.locals[self.local_count - 1].depth = self.scope_depth;
    }

    fn function(&mut self) -> &mut ObjFunction {
        unsafe { self.function.as_mut().unwrap() }
    }
    fn enclosing(&mut self) -> Option<&mut Compiler> {
        unsafe { self.enclosing.as_mut() }
    }
}

#[derive(Clone, Copy, Debug)]
struct Local {
    name: Token,
    depth: isize,
    is_captured: bool,
}

impl Local {
    fn null() -> Local {
        let name = Token {
            id: TokenType::EOF,
            start: std::ptr::null(),
            length: 0,
            line: 0,
        };
        Local {
            name,
            depth: -1,
            is_captured: false,
        }
    }
}
#[derive(Clone, Copy, Debug)]
struct Upvalue {
    index: isize,
    is_local: bool,
}

impl Upvalue {
    fn new() -> Self {
        Self {
            index: -1,
            is_local: false,
        }
    }
}
struct Parser<'b> {
    previous: Token,
    current: Token,
    had_error: Result<()>,
    scanner: &'b mut Scanner<'b>,
}

impl<'b> Parser<'b> {
    fn new(scanner: &'b mut Scanner<'b>, compiler: &mut Compiler) -> Parser<'b> {
        let null = scanner::Token {
            id: TokenType::Error,
            start: ptr::null(),
            length: 0,
            line: 0,
        };
        unsafe {
            COMPILER = compiler;
        }
        Parser {
            previous: null,
            current: null,
            scanner,
            had_error: Ok(()),
        }
    }
    fn compiler(&mut self) -> &mut Compiler {
        unsafe { COMPILER.as_mut().unwrap() }
    }
    fn current_chunk(&mut self) -> &mut Chunk {
        let function = self.compiler().function();
        &mut function.chunk
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

    fn error_at<T: Into<String>>(parser: &mut Parser, token: Token, message: T) -> Result<()> {
        let mut error = format!("[line {}] Error", token.line);
        if token.id == TokenType::EOF {
            error.push_str(" at end");
        } else {
            error.push_str(" at '");
            error.push_str(token.extract());
            error.push('\'');
        }
        error.push_str(": ");
        error.push_str(&message.into());
        if let Err(_) = parser.had_error {
            Err(Error(error))
        } else {
            eprintln!("{}", error);
            parser.had_error = Err(Error(error));
            parser.had_error.clone()
        }
    }

    fn error_at_current<T: Into<String>>(&mut self, message: T) -> Result<()> {
        Self::error_at(self, self.current, message)
    }

    fn error<T: Into<String>>(&mut self, message: T) -> Result<()> {
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
            self.error(message)
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
        self.current_chunk().write(byte, line);
    }

    fn emit_bytes<T: Into<U>, U: Into<u8>>(&mut self, byte1: T, byte2: U) {
        self.emit_byte(byte1.into());
        self.emit_byte(byte2);
    }

    fn emit_loop(&mut self, loop_start: usize) -> Result<()> {
        self.emit_byte(OpCode::Loop);

        let offset = self.current_chunk().code.len() - loop_start + 2;
        if offset > u16::MAX as usize {
            return self.error("Loop body too large.");
        }

        self.emit_byte((((offset as u16) >> 8) & 0xff) as u8);
        self.emit_byte((offset as u8) & 0xff);
        Ok(())
    }
    fn emit_jump<T: Into<u8>>(&mut self, instruction: T) -> usize {
        self.emit_byte(instruction);
        self.emit_byte(0xff);
        self.emit_byte(0xff);
        self.current_chunk().code.len() - 2
    }

    fn patch_jump(&mut self, offset: usize) -> Result<()> {
        let jump = self.current_chunk().code.len() - offset - 2;
        if jump > u16::MAX as usize {
            return self.error("Too much code to jump over.");
        }
        let jump = jump as u16;
        self.current_chunk().code[offset as usize] = ((jump >> 8) & 0xff) as u8;
        self.current_chunk().code[offset + 1] = (jump & 0xff) as u8;

        Ok(())
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Nil);
        self.emit_byte(OpCode::Return);
    }

    fn emit_constant<T: Into<Value>>(&mut self, value: T) {
        let n = self.current_chunk().constant(value.into());
        self.emit_bytes(OpCode::Constant, n);
    }

    fn make_constant<T: Into<Value>>(&mut self, value: T) -> u8 {
        self.current_chunk().constant(value.into())
    }
    fn end_compiler(&mut self) -> (*mut Compiler, *mut ObjFunction) {
        self.emit_return();
        let function = self.compiler().function();
        if function.name.is_null() {
            self.current_chunk().set_name("script");
        } else {
            let name = unsafe { function.name.as_ref().unwrap() };
            self.current_chunk().set_name(name.as_rstring());
        }
        #[cfg(feature = "print_code")]
        if self.had_error.is_ok() {
            println!("{:?}", self.current_chunk());
        }
        let function = self.compiler().function;
        let compiler = unsafe { COMPILER };
        unsafe { COMPILER = self.compiler().enclosing };
        (compiler, function)
    }

    fn begin_scope(&mut self) {
        self.compiler().scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.compiler().scope_depth -= 1;

        while self.compiler().local_count > 0 && {
            let local_count = self.compiler().local_count - 1;
            self.compiler().locals[local_count].depth
        } > self.compiler().scope_depth
        {
            let local_count = self.compiler().local_count - 1;
            if self.compiler().locals[local_count].is_captured {
                self.emit_byte(OpCode::CloseUpvalue);
            } else {
                self.emit_byte(OpCode::Pop);
            }
            self.compiler().local_count -= 1;
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
        let string = allocate_string(name.extract()) as *mut dyn Obj;
        self.current_chunk().constant(string.into())
    }

    fn declare_variable(&mut self) -> CmpResult<()> {
        if self.compiler().scope_depth == 0 {
            return Ok(());
        }

        let name = self.previous;

        if self.compiler().local_count > 0 {
            for i in (0..=(self.compiler().local_count - 1)).rev() {
                let scope_depth = self.compiler().scope_depth;
                let local = &self.compiler().locals[i as usize];
                if local.depth != -1 && local.depth < scope_depth {
                    break;
                }

                if name.extract() == local.name.extract() {
                    self.error("Already a variable with this name in this scope.")?;
                }
            }
        }

        self.compiler().add_local(name)
    }

    fn parse_variable(&mut self, error_message: &str) -> Result<u8> {
        self.consume(TokenType::Identifier, error_message)?;
        if let Err(err) = self.declare_variable() {
            self.error(err)?;
        }
        if self.compiler().scope_depth > 0 {
            return Ok(0);
        }
        Ok(self.identifier_constant(self.previous))
    }

    fn define_variable(&mut self, global: u8) {
        if self.compiler().scope_depth > 0 {
            self.compiler().mark_initialized();
            return;
        }
        self.emit_bytes(OpCode::DefineGlobal, global);
    }

    fn argument_list(&mut self) -> Result<u8> {
        let mut arg_count = 0;
        if !self.check(TokenType::RightParen) {
            loop {
                expression(self)?;
                if arg_count == 255 {
                    self.error("Can't have more than 255 arguments.")?;
                }
                arg_count += 1;
                if !self.matches(TokenType::Comma)? {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
        Ok(arg_count)
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) -> Result<()> {
        let (arg, get_opt, set_opt) = match self.compiler().resolve_local(name)? {
            Some(n) => (n, OpCode::GetLocal, OpCode::SetLocal),
            None => match self.compiler().resolve_upvalue(name)? {
                Some(n) => (n, OpCode::GetUpvalue, OpCode::SetUpvalue),
                None => (
                    self.identifier_constant(name),
                    OpCode::GetGlobal,
                    OpCode::SetGlobal,
                ),
            },
        };

        let code = if can_assign && self.matches(TokenType::Equal).unwrap() {
            expression(self)?;
            set_opt
        } else {
            get_opt
        };
        self.emit_bytes(code, arg);
        Ok(())
    }
}

use compiler_functions::*;

mod compiler_functions {
    use super::{rule::get_rule, Compiler, FunctionType, Parser, Precedence, Result, COMPILER};
    use crate::{
        chunk::OpCode,
        objects::{allocate_string, Obj},
        scanner::TokenType,
        vm::Vm,
    };

    pub(super) fn r#and(parser: &mut Parser, _: bool) -> Result<()> {
        let end_jump = parser.emit_jump(OpCode::JumpIfFalse);
        parser.emit_byte(OpCode::Pop);
        parser.parse_precedence(Precedence::And)?;

        parser.patch_jump(end_jump)
    }
    pub(super) fn number(parser: &mut Parser, _: bool) -> Result<()> {
        let value = parser.previous.extract().parse::<f64>().unwrap();
        parser.emit_constant(value);
        Ok(())
    }

    pub(super) fn r#or(parser: &mut Parser, _: bool) -> Result<()> {
        let else_jump = parser.emit_jump(OpCode::JumpIfFalse);
        let end_jump = parser.emit_jump(OpCode::Jump);

        parser.patch_jump(else_jump)?;
        parser.emit_byte(OpCode::Pop);

        parser.parse_precedence(Precedence::Or)?;
        parser.patch_jump(end_jump)
    }

    pub(super) fn string(parser: &mut Parser, _: bool) -> Result<()> {
        let string = allocate_string(parser.previous.extract()) as *mut dyn Obj;
        parser.emit_constant(string);
        Ok(())
    }

    pub(super) fn variable(parser: &mut Parser, can_assign: bool) -> Result<()> {
        parser.named_variable(parser.previous, can_assign)
    }

    pub(super) fn grouping(parser: &mut Parser, _: bool) -> Result<()> {
        match expression(parser) {
            _ => parser.consume(TokenType::RightParen, "Expect ')' after expression"),
        }
    }

    pub(super) fn unary(parser: &mut Parser, _: bool) -> Result<()> {
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

    pub(super) fn binary(parser: &mut Parser, _: bool) -> Result<()> {
        let op_type = parser.previous.id;
        let rule = get_rule(op_type);
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

    pub(super) fn call(parser: &mut Parser, _: bool) -> Result<()> {
        let arg_count = parser.argument_list()?;
        parser.emit_bytes(OpCode::Call, arg_count);
        Ok(())
    }

    pub(super) fn literal(parser: &mut Parser, _: bool) -> Result<()> {
        parser.emit_byte(match parser.previous.id {
            TokenType::False => OpCode::False,
            TokenType::Nil => OpCode::Nil,
            TokenType::True => OpCode::True,
            _ => unreachable!(),
        });
        Ok(())
    }

    pub(super) fn expression(parser: &mut Parser) -> Result<()> {
        parser.parse_precedence(Precedence::Assignment)
    }

    pub(super) fn block(parser: &mut Parser) -> Result<()> {
        while !parser.check(TokenType::RightBrace) && !parser.check(TokenType::EOF) {
            declaration(parser);
        }
        parser.consume(TokenType::RightBrace, "Expect '}' after block.")
    }
    pub(super) fn function(parser: &mut Parser, id: FunctionType) -> Result<()> {
        let string = allocate_string(parser.previous.extract());
        let string = unsafe { string.as_mut().unwrap().as_string_mut().unwrap() };
        let mut compiler = Compiler::new(id, unsafe { COMPILER }, string);
        unsafe { COMPILER = &mut compiler };
        parser.begin_scope();

        parser.consume(TokenType::LeftParen, "Expect '(' after funciton name.")?;
        if !parser.check(TokenType::RightParen) {
            loop {
                parser.compiler().function().arity += 1;
                if parser.compiler().function().arity > 255 {
                    parser.error_at_current("Can't have more than 255 parameters.")?;
                }

                let constant = parser.parse_variable("Expect parameter name.")?;
                parser.define_variable(constant);

                if !parser.matches(TokenType::Comma)? {
                    break;
                }
            }
        }
        parser.consume(TokenType::RightParen, "Expect ')' after paramaters.")?;
        parser.consume(TokenType::LeftBrace, "Expect '{' befor funciton body.")?;
        block(parser)?;
        let (compiler, function) = parser.end_compiler();
        Vm::push(function as *mut dyn Obj);
        let constant = parser.make_constant(function as *mut dyn Obj);
        Vm::pop();
        parser.emit_bytes(OpCode::Closure, constant);
        for i in 0..unsafe {
            function
                .as_ref()
                .expect("Uninitialized function")
                .upvalue_count as usize
        } {
            let local = if unsafe { compiler.as_ref().unwrap() }.upvalues[i].is_local {
                1
            } else {
                0
            };
            parser.emit_byte(local);
            let index = unsafe { compiler.as_ref().unwrap() }.upvalues[i].index;
            parser.emit_byte(index as u8);
        }
        Ok(())
    }
    pub(super) fn fun_declaration(parser: &mut Parser) -> Result<()> {
        let global = parser.parse_variable("Expect function name.")?;
        parser.compiler().mark_initialized();
        function(parser, FunctionType::Function)?;
        parser.define_variable(global);
        Ok(())
    }

    pub(super) fn var_declaration(parser: &mut Parser) -> Result<()> {
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

    pub(super) fn expression_statement(parser: &mut Parser) -> Result<()> {
        expression(parser)?;
        parser.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        parser.emit_byte(OpCode::Pop);
        Ok(())
    }
    pub(super) fn for_statement(parser: &mut Parser) -> Result<()> {
        parser.begin_scope();
        parser.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;
        if parser.matches(TokenType::Semicolon)? {
            // No initializer.
        } else if parser.matches(TokenType::Var)? {
            var_declaration(parser)?;
        } else {
            expression_statement(parser)?;
        }

        let mut loop_start = parser.current_chunk().code.len();
        let mut exit_jump = None;
        if !parser.matches(TokenType::Semicolon)? {
            expression(parser)?;
            parser.consume(TokenType::Semicolon, "Expect ';' after loop  condition.")?;

            // Jump out of the loop if the condition is false.
            exit_jump = Some(parser.emit_jump(OpCode::JumpIfFalse));
            parser.emit_byte(OpCode::Pop);
        }

        if !parser.matches(TokenType::RightParen)? {
            let body_jump = parser.emit_jump(OpCode::Jump);
            let increment_start = parser.current_chunk().code.len();
            expression(parser)?;
            parser.emit_byte(OpCode::Pop);
            parser.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

            parser.emit_loop(loop_start)?;
            loop_start = increment_start;
            parser.patch_jump(body_jump)?;
        }

        statement(parser)?;
        parser.emit_loop(loop_start)?;

        if let Some(exit_jump) = exit_jump {
            parser.patch_jump(exit_jump)?;
            parser.emit_byte(OpCode::Pop);
        }
        parser.end_scope();
        Ok(())
    }
    pub(super) fn if_statement(parser: &mut Parser) -> Result<()> {
        parser.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        expression(parser)?;
        parser.consume(TokenType::RightParen, "Expect ')' after condition.")?;

        let then_jump = parser.emit_jump(OpCode::JumpIfFalse);
        parser.emit_byte(OpCode::Pop);
        statement(parser)?;

        let else_jump = parser.emit_jump(OpCode::Jump);
        parser.patch_jump(then_jump)?;
        parser.emit_byte(OpCode::Pop);

        if parser.matches(TokenType::Else)? {
            statement(parser)?;
        }
        parser.patch_jump(else_jump)
    }

    pub(super) fn print_statement(parser: &mut Parser) -> Result<()> {
        expression(parser)?;
        parser.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        parser.emit_byte(OpCode::Print);
        Ok(())
    }
    pub(super) fn return_statement(parser: &mut Parser) -> Result<()> {
        if parser.compiler().id == FunctionType::Script {
            parser.error("Can't return from top-level code.")?;
        }
        if parser.matches(TokenType::Semicolon)? {
            parser.emit_return();
            Ok(())
        } else {
            expression(parser)?;
            parser.consume(TokenType::Semicolon, "Expect ';' after return value.")?;
            parser.emit_byte(OpCode::Return);
            Ok(())
        }
    }
    pub(super) fn while_statement(parser: &mut Parser) -> Result<()> {
        let loop_start = parser.current_chunk().code.len();
        parser.consume(TokenType::LeftParen, "Expect '(' after'while'.")?;
        expression(parser)?;
        parser.consume(TokenType::RightParen, "Expect ')' after condition.")?;

        let exit_jump = parser.emit_jump(OpCode::JumpIfFalse);
        parser.emit_byte(OpCode::Pop);
        statement(parser)?;
        parser.emit_loop(loop_start)?;

        parser.patch_jump(exit_jump)?;
        parser.emit_byte(OpCode::Pop);
        Ok(())
    }
    pub(super) fn declaration(parser: &mut Parser) {
        let n = if let Ok(true) = parser.matches(TokenType::Fun) {
            fun_declaration(parser)
        } else if let Ok(true) = parser.matches(TokenType::Var) {
            var_declaration(parser)
        } else {
            statement(parser)
        };
        if n.is_err() {
            parser.syncronize();
        }
    }

    pub(super) fn statement(parser: &mut Parser) -> Result<()> {
        if parser.matches(TokenType::Print)? {
            print_statement(parser)
        } else if parser.matches(TokenType::For)? {
            for_statement(parser)
        } else if parser.matches(TokenType::If)? {
            if_statement(parser)
        } else if parser.matches(TokenType::Return)? {
            return_statement(parser)
        } else if parser.matches(TokenType::While)? {
            while_statement(parser)
        } else if parser.matches(TokenType::LeftBrace)? {
            parser.begin_scope();
            let n = block(parser);
            parser.end_scope();
            n
        } else {
            expression_statement(parser)
        }
    }
}

pub fn compile(source: &str) -> Result<*mut ObjFunction> {
    let mut compiler = Compiler::new(FunctionType::Script, ptr::null_mut(), ptr::null_mut());
    let mut scanner = Scanner::new(source);
    let mut parser = Parser::new(&mut scanner, &mut compiler);
    parser.advance()?;
    while !parser.matches(TokenType::EOF)? {
        declaration(&mut parser);
    }
    let n = parser.end_compiler();
    if let Err(err) = parser.had_error {
        Err(err)
    } else {
        Ok(n.1)
    }
}

pub unsafe fn mark_compiler_roots() {
    let mut compiler = COMPILER;
    while !compiler.is_null() {
        let com = compiler.as_mut().unwrap();
        GarbageCollector::mark_obj(com.function);
        compiler = com.enclosing;
    }
}
mod rule {
    macro_rules! define {
        ($_:ty, $prefix:expr, $infix:expr, $precedence:expr) => {
            ParseRule {
                prefix: $prefix,
                infix: $infix,
                precedence: $precedence,
            }
        };
    }
    #[rustfmt::skip]
    const RULES: [ParseRule; 40] = [
        define!(TokenType::LeftParen   , Some(grouping), Some(call)  , Precedence::Call      ),
        define!(TokenType::RightParen  , None          , None        , Precedence::None      ),
        define!(TokenType::LeftBrace   , None          , None        , Precedence::None      ),
        define!(TokenType::RightBrace  , None          , None        , Precedence::None      ),
        define!(TokenType::Comma       , None          , None        , Precedence::None      ),
        define!(TokenType::Dot         , None          , None        , Precedence::None      ),
        define!(TokenType::Minus       , Some(unary)   , Some(binary), Precedence::Term      ),
        define!(TokenType::Plus        , None          , Some(binary), Precedence::Term      ),
        define!(TokenType::Semicolon   , None          , None        , Precedence::None      ),
        define!(TokenType::Slash       , None          , Some(binary), Precedence::Factor    ),
        define!(TokenType::Star        , None          , Some(binary), Precedence::Factor    ),
        define!(TokenType::Bang        , Some(unary)   , None        , Precedence::None      ),
        define!(TokenType::BangEqual   , None          , Some(binary), Precedence::Equality  ),
        define!(TokenType::Equal       , None          , None        , Precedence::None      ),
        define!(TokenType::EqualEqual  , None          , Some(binary), Precedence::Equality  ),
        define!(TokenType::Greater     , None          , Some(binary), Precedence::Comparison),
        define!(TokenType::GreaterEqual, None          , Some(binary), Precedence::Comparison),
        define!(TokenType::Less        , None          , Some(binary), Precedence::Comparison),
        define!(TokenType::LessEqual   , None          , Some(binary), Precedence::Comparison),
        define!(TokenType::Identifier  , Some(variable), None        , Precedence::None      ),
        define!(TokenType::String      , Some(string)  , None        , Precedence::None      ),
        define!(TokenType::Number      , Some(number)  , None        , Precedence::None      ),
        define!(TokenType::And         , None          , Some(r#and) , Precedence::And       ),
        define!(TokenType::Class       , None          , None        , Precedence::None      ),
        define!(TokenType::Else        , None          , None        , Precedence::None      ),
        define!(TokenType::False       , Some(literal) , None        , Precedence::None      ),
        define!(TokenType::For         , None          , None        , Precedence::None      ),
        define!(TokenType::Fun         , None          , None        , Precedence::None      ),
        define!(TokenType::If          , None          , None        , Precedence::None      ),
        define!(TokenType::Nil         , Some(literal) , None        , Precedence::None      ),
        define!(TokenType::Or          , None          , Some(r#or)  , Precedence::Or        ),
        define!(TokenType::Print       , None          , None        , Precedence::None      ),
        define!(TokenType::Return      , None          , None        , Precedence::None      ),
        define!(TokenType::Super       , None          , None        , Precedence::None      ),
        define!(TokenType::This        , None          , None        , Precedence::None      ),
        define!(TokenType::True        , Some(literal) , None        , Precedence::None      ),
        define!(TokenType::Var         , None          , None        , Precedence::None      ),
        define!(TokenType::While       , None          , None        , Precedence::None      ),
        define!(TokenType::Error       , None          , None        , Precedence::None      ),
        define!(TokenType::EOF         , None          , None        , Precedence::None      ),
    ];
    use super::{
        binary, call, grouping, literal, number, r#and, r#or, string, unary, variable, Parser,
    };
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
