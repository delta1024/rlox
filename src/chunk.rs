use std::{
    fmt::{Debug, Display},
    pin::Pin,
};

use crate::value::Value;
#[derive(Debug)]
pub struct Lines {
    /// (line number, count)
    code: Vec<(usize, usize)>,
}

impl Lines {
    pub fn new() -> Self {
        Self { code: Vec::new() }
    }

    pub fn add_line(&mut self, new_line: usize) {
        for (line, count) in &mut self.code {
            if *line == new_line {
                *count += 1;
                return;
            }
        }

        self.code.push((new_line, 1));
    }

    pub fn get_line(&self, mut pos: usize) -> Option<usize> {
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

pub struct Chunk {
    code: Vec<u8>,
    lines: Lines,
    constants: Vec<Value>,
    _name: [char; 250],
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            lines: Lines::new(),
            constants: Vec::new(),
            _name: ['\0'; 250],
        }
    }

    pub fn set_name(&mut self, _name: &str) {
        todo!()
    }
    pub fn name(&self) -> String {
        String::from("test chunk")
    }
    pub fn write<T: Into<u8>>(&mut self, code: T, line: usize) {
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

    pub fn ip(self: Pin<&Self>) -> Ip {
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
        let mut out = format!("== {} ==\n", self.name());

        let mut line = 0;
        let mut ip = Ip::new(self).enumerate();
        loop {
            let (off, i) = match ip.next() {
                Some(byte) => byte,
                None => break,
            };
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

            match i.into() {
                OpCode::Constant => {
                    let (_, n) = ip.next().unwrap();
                    out.push_str(&format!(
                        "{}{:<10} {} '{}'\n",
                        OpCode::from(i),
                        " ",
                        n,
                        unsafe { Ip::new(self).get_constant(n) }
                    ));
                }

                OpCode::Return => {
                    out.push_str(&format!("{}", OpCode::from(i)));
                }
            }
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


pub struct Ip {
    head: *const u8,
    tail: *const u8,
    current: *const u8,
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
    pub fn disassemble_instruction(&self) -> String {
        let offset = unsafe { self.current.offset_from(self.head) };

        let op: OpCode = unsafe { self.current.sub(1).read().into() };
        match op {
            OpCode::Constant => {
                let n = self.peek().unwrap();
                let m = unsafe { self.get_constant(n) };
                format!("{:04} {} {:<9}{} '{}'", offset, op, " ", n, m)
            }
            OpCode::Return => format!("{:04} {}", offset, op),
        }
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

#[derive(Debug)]
pub enum OpCode {
    Return,
    Constant,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Return => "OP_RETURN",
                Self::Constant => "OP_CONSTANT",
            }
        )
    }
}

impl From<OpCode> for u8 {
    fn from(code: OpCode) -> Self {
        match code {
            OpCode::Return => 0,
            OpCode::Constant => 1,
        }
    }
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {
        match byte {
            0 => OpCode::Return,
            1 => OpCode::Constant,
            _ => panic!("Unrecongnised OpCode: {}", byte),
        }
    }
}