use std::{fmt::Debug, pin::Pin, ptr};

use crate::value::Value;
#[derive(Debug)]
pub struct Lines {
    /// (line number, count)
    code: Vec<(u32, u32)>,
}

impl Lines {
    pub fn new() -> Self {
        Self { code: Vec::new() }
    }

    pub fn add_line(&mut self, new_line: u32) {
        for (line, count) in &mut self.code {
            if *line == new_line {
                *count += 1;
                return;
            }
        }

        self.code.push((new_line, 1));
    }

    pub fn get_line(&self, mut pos: u32) -> Option<u32> {
        for (line, count) in &self.code {
            if pos > *count {
                pos -= *count;
            } else {
                return Some(*line);
            }
        }
        None
    }
}

const NAME_LEN: usize = 250;
pub struct Chunk {
    pub code: Vec<u8>,
    lines: Lines,
    constants: Vec<Value>,
    name: [u8; NAME_LEN],
}
impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            lines: Lines::new(),
            constants: Vec::new(),
            name: ['\0' as u8; NAME_LEN],
        }
    }

    pub fn set_name(&mut self, name: &str) {
        for (i, c) in name.chars().enumerate() {
            self.name[i] = c as u8;
        }
    }
    pub fn get_name(&self) -> &str {
        let mut len = 0;
        while self.name[len] as char != '\0' {
            len += 1;
        }

        let slice = &self.name[..len];
        std::str::from_utf8(slice).unwrap()
    }
    pub fn clear_name(&mut self) {
        for i in 0..NAME_LEN {
            if self.name[i] == b'\0' {
                return;
            }
            self.name[i] = b'\0';
        }
    }
    pub fn write<T: Into<u8>>(&mut self, code: T, line: u32) {
        self.code.push(code.into());
        self.lines.add_line(line);
    }

    pub fn constant(&mut self, val: Value) -> u8 {
        self.constants.push(val);
        (self.constants.len() - 1) as u8
    }

    pub fn pin(self) -> Pin<Box<Self>> {
        Box::pin(self)
    }

    pub fn ip(&self) -> Ip {
        Ip {
            head: self.code.as_ptr(),
            tail: unsafe { self.code.as_ptr().add(self.code.len()) },
            current: self.code.as_ptr(),
            lines: &self.lines,
            constants: self.constants.as_ptr(),
        }
    }
}
impl Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = format!("== {} ==\n", self.get_name());

        let mut line = 0;
        let mut ip = Ip::new(self);
        loop {
            let off = ip.offset();
            if ip.next().is_none() {
                break;
            }
            out.push_str(&format!("{:04} ", off));
            {
                let last_line = if off != 0 {
                    self.lines.get_line(off - 1)
                } else {
                    self.lines.get_line(off)
                }
                .unwrap();
                if last_line != line {
                    line = self.lines.get_line(off).unwrap();
                    out.push_str(&format!("{:04} ", line));
                } else {
                    out.push_str("   | ");
                }
            }
            let (instruction, skip) = ip.disassemble_instruction();
            for _ in 0..skip {
                ip.next();
            }
            out.push_str(&instruction);
            out.push('\n');
        }
        write!(f, "{}", out)
    }
}
impl IntoIterator for &Chunk {
    type Item = u8;
    type IntoIter = Ip;
    fn into_iter(self) -> Self::IntoIter {
        Ip::new(self)
    }
}
#[derive(Clone, Copy)]
/// An abstraction over all unsafe code related to a Chunk
pub struct Ip {
    head: *const u8,
    tail: *const u8,
    pub current: *const u8,
    lines: *const Lines,
    constants: *const Value,
}

impl Ip {
    fn new(chunk: &Chunk) -> Self {
        Ip {
            head: chunk.code.as_ptr(),
            tail: unsafe { chunk.code.as_ptr().add(chunk.code.len()) },
            current: chunk.code.as_ptr(),
            lines: &chunk.lines,
            constants: chunk.constants.as_ptr(),
        }
    }
    pub const fn null() -> Self {
        Self {
            head: ptr::null(),
            tail: ptr::null(),
            current: ptr::null(),
            lines: ptr::null(),
            constants: ptr::null(),
        }
    }
    pub unsafe fn get_constant(&self, pos: u8) -> Value {
        self.constants.add(pos as usize).read()
    }
    pub unsafe fn get_lines(&self) -> &Lines {
        self.lines.as_ref().unwrap()
    }
    pub fn previous(&self) -> Option<u8> {
        if self.head == self.current {
            return None;
        }

        unsafe { Some(self.current.sub(1).read()) }
    }
    pub fn peek(&self) -> Option<u8> {
        if self.current == self.tail {
            return None;
        }

        unsafe { Some(self.current.read()) }
    }
    pub fn disassemble_instruction(&self) -> (String, usize) {
        let offset = unsafe { self.current.offset_from(self.head) };

        let op: OpCode = unsafe { self.current.sub(1).read().into() };
        match op {
            OpCode::Constant | OpCode::DefineGlobal | OpCode::GetGlobal | OpCode::SetGlobal => {
                let n = self.peek().unwrap();
                let m = unsafe { self.get_constant(n) };
                (format!("{} {:<9}{} '{}'", op, " ", n, m), 1)
            }
            OpCode::Jump | OpCode::JumpIfFalse => {
                let mut jump = unsafe { (self.current.read() as u16) << 8 };
                jump |= unsafe { self.current.add(1).read() as u16 };
                (
                    format!(
                        "{} {} {:04} -> {}",
                        op,
                        match op {
                            OpCode::JumpIfFalse => format! {"{:<2} ", " "},
                            OpCode::Jump => format!("{:<11} ", " "),
                            _ => unreachable!(),
                        },
                        offset,
                        (offset + 3 + (1 * jump) as isize)
                    ),
                    2,
                )
            }
            OpCode::Closure => {
                let mut offset = offset + 1;
                let constant = unsafe {
                    self.head
                        .add({
                            offset += 1;
                            (offset - 1) as usize
                        })
                        .read()
                };
                unsafe {
                    (
                        format!(
                            "{} {:<11} {:04} {}\n",
                            op,
                            " ",
                            constant,
                            self.get_constant(constant)
                        ),
                        offset as usize,
                    )
                }
            }
            OpCode::Loop => {
                let mut jump = unsafe { (self.current.read() as u16) << 8 };
                jump |= unsafe { self.current.add(1).read() as u16 };
                (
                    format!(
                        "{} {:<11} {:04} -> {}",
                        op,
                        " ",
                        offset,
                        (offset + 3 + (-1 * jump as isize) as isize)
                    ),
                    2,
                )
            }
            OpCode::GetLocal | OpCode::SetLocal | OpCode::Call => {
                let slot = self.peek().unwrap();
                (format! {"{} {:<5} {}", op, " ", slot}, 1)
            }
            _ => (format!("{}", op), 0),
        }
    }
    pub fn offset(&self) -> u32 {
        unsafe { self.current.offset_from(self.head) as u32 }
    }

    pub fn jump_forward(&mut self, span: usize) {
        self.current = unsafe { self.current.add(span) };
    }
    pub fn jump_back(&mut self, span: usize) {
        self.current = unsafe { self.current.sub(span) };
    }
    pub fn short_bytes(&mut self) -> (u8, u8) {
        unsafe { (self.current.sub(2).read(), self.current.sub(1).read()) }
    }
}

impl Iterator for Ip {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.tail {
            return None;
        }
        unsafe {
            self.current = self.current.add(1);
            Some(self.current.sub(1).read())
        }
    }
}

pub use opcode::*;
mod opcode {
    use std::fmt::Display;
    #[derive(Debug)]
    pub enum OpCode {
        Return,
        Constant,
        Negate,
        Add,
        Subtract,
        Multiply,
        Divide,
        Nil,
        True,
        False,
        Not,
        Equal,
        Greater,
        Less,
        Print,
        Pop,
        DefineGlobal,
        GetGlobal,
        SetGlobal,
        GetLocal,
        SetLocal,
        JumpIfFalse,
        Jump,
        Loop,
        Call,
        Closure,
    }
    macro_rules! from_and_into {
        ( $( $code: tt, $name: tt, $value: literal),*) => {
            impl From<OpCode> for u8 {
                fn from(value: OpCode) -> Self {

                    match value {
                        $(
                            OpCode::$code => $value,
                        )*
                    }
                }
            }

            impl From<u8> for OpCode {
                fn from(op_code: u8) -> Self {
                    match op_code {
                        $(
                            $value => Self::$code,
                        )*
                        _ => panic!("Unrecongnised OpCode: {}", op_code)                    }
                }
            }

            impl Display for OpCode {

                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(
                        f,
                        "OP_{}",
                        match self {
                            $(
                                Self::$code => $name,
                            )*
                        })
                }
            }
        };
    }
    from_and_into!(
        Return,
        "RETURN",
        0,
        Constant,
        "CONSTANT",
        1,
        Negate,
        "NEGATE",
        2,
        Add,
        "ADD",
        3,
        Subtract,
        "SUBTRACT",
        4,
        Multiply,
        "MULTIPLY",
        5,
        Divide,
        "DIVIDE",
        6,
        Nil,
        "NIL",
        7,
        True,
        "TRUE",
        8,
        False,
        "FALSE",
        9,
        Not,
        "NOT",
        10,
        Equal,
        "EQUAL",
        11,
        Greater,
        "GREATER",
        12,
        Less,
        "LESS",
        13,
        Print,
        "PRINT",
        14,
        Pop,
        "POP",
        15,
        DefineGlobal,
        "DEFINEGLOBAL",
        16,
        GetGlobal,
        "GETGLOBAL",
        17,
        SetGlobal,
        "SETGLOBAL",
        18,
        GetLocal,
        "GETLOCAL",
        19,
        SetLocal,
        "SETLOCAL",
        20,
        JumpIfFalse,
        "JUMPIFFALSE",
        21,
        Jump,
        "JUMP",
        22,
        Loop,
        "LOOP",
        23,
        Call,
        "CALL",
        24,
        Closure,
        "CLOSURE",
        25
    );
}
