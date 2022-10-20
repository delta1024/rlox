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
#[cfg(feature = "debug")]
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
    #[cfg(feature = "debug")]
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
    }

    impl Display for OpCode {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "OP_{}",
                match self {
                    Self::Return => "RETURN",
                    Self::Constant => "CONSTANT",
                    Self::Negate => "NEGATE",
                    Self::Add => "ADD",
                    Self::Subtract => "SUBTRACT",
                    Self::Multiply => "MULTIPLY",
                    Self::Divide => "DIVIDE",
                    Self::Nil => "NIL",
                    Self::True => "TRUE",
                    Self::False => "False",
                    Self::Not => "NOT",
                    Self::Equal => "EQUAL",
                    Self::Greater => "GREATER",
                    Self::Less => "LESS",
                    Self::Print => "PRINT",
                    Self::Pop => "POP",
                    Self::DefineGlobal => "DEFINE_GLOBAL",
                    Self::GetGlobal => "GET_GLOBAL",
                    Self::SetGlobal => "SET_GLOBAL",
                    Self::GetLocal => "GET_LOCAL",
                    Self::SetLocal => "SET_LOCAL",
                    Self::JumpIfFalse => "JUMP_IF_FALSE",
                    Self::Jump => "JUMP",
                    Self::Loop => "LOOP",
                    OpCode::Call => "CALL",
                }
            )
        }
    }

    impl From<OpCode> for u8 {
        fn from(code: OpCode) -> Self {
            match code {
                OpCode::Return => 0,
                OpCode::Constant => 1,
                OpCode::Negate => 2,
                OpCode::Add => 3,
                OpCode::Subtract => 4,
                OpCode::Multiply => 5,
                OpCode::Divide => 6,
                OpCode::Nil => 7,
                OpCode::True => 8,
                OpCode::False => 9,
                OpCode::Not => 10,
                OpCode::Equal => 11,
                OpCode::Greater => 12,
                OpCode::Less => 13,
                OpCode::Print => 14,
                OpCode::Pop => 15,
                OpCode::DefineGlobal => 16,
                OpCode::GetGlobal => 17,
                OpCode::SetGlobal => 18,
                OpCode::GetLocal => 19,
                OpCode::SetLocal => 20,
                OpCode::JumpIfFalse => 21,
                OpCode::Jump => 22,
                OpCode::Loop => 23,
                OpCode::Call => 24,
            }
        }
    }

    impl From<u8> for OpCode {
        fn from(byte: u8) -> Self {
            match byte {
                0 => OpCode::Return,
                1 => OpCode::Constant,
                2 => OpCode::Negate,
                3 => OpCode::Add,
                4 => OpCode::Subtract,
                5 => OpCode::Multiply,
                6 => OpCode::Divide,
                7 => OpCode::Nil,
                8 => OpCode::True,
                9 => OpCode::False,
                10 => OpCode::Not,
                11 => OpCode::Equal,
                12 => OpCode::Greater,
                13 => OpCode::Less,
                14 => OpCode::Print,
                15 => OpCode::Pop,
                16 => OpCode::DefineGlobal,
                17 => OpCode::GetGlobal,
                18 => OpCode::SetGlobal,
                19 => OpCode::GetLocal,
                20 => OpCode::SetLocal,
                21 => OpCode::JumpIfFalse,
                22 => OpCode::Jump,
                23 => OpCode::Loop,
                24 => OpCode::Call,
                _ => panic!("Unrecongnised OpCode: {}", byte),
            }
        }
    }
}
