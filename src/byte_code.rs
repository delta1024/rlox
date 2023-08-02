use crate::{frame::PositionCounter, value::Value};
use std::fmt;
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Internal Operation Code to be exacuted.
pub(crate) enum OpCode {
    Return,
    Constant(Value),
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    Print,
}
impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        match value {
            OpCode::Return => 0,
            OpCode::Constant(_) => 1,
            OpCode::Add => 2,
            OpCode::Sub => 3,
            OpCode::Mul => 4,
            OpCode::Div => 5,
            OpCode::Neg => 6,
            OpCode::Print => 7,
        }
    }
}
impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Return,
            2 => Self::Add,
            3 => Self::Sub,
            4 => Self::Mul,
            5 => Self::Div,
            6 => Self::Neg,
            7 => Self::Print,
            _ => unreachable!(),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ReadInstructionError {
    InvaildInstruction,
}
impl std::error::Error for ReadInstructionError {}
impl fmt::Display for ReadInstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid opcode")
    }
}
#[derive(Debug)]
pub(crate) struct Chunk {
    code: Box<[u8]>,
    values: Box<[Value]>,
    lines: Box<[u8]>,
}
impl Chunk {
    pub fn read_instruction(
        &self,
        position: PositionCounter,
    ) -> Result<(OpCode, usize), ReadInstructionError> {
        use OpCode::*;
        let position = position.0 as usize;
        match self.code[position] {
            0 | 2..=7 => Ok((self.code[position].into(), position + 1)),
            1 => Ok((
                Constant(self.values[self.code[position + 1] as usize]),
                position + 2,
            )),
            _ => Err(ReadInstructionError::InvaildInstruction),
        }
    }
}
#[derive(Default, Debug)]
pub(crate) struct ChunkBuilder {
    code: Vec<u8>,
    values: Vec<Value>,
    lines: Vec<u8>,
}
impl ChunkBuilder {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    pub(crate) fn write_byte(&mut self, op_code: OpCode, line: u8) {
        match op_code {
            OpCode::Return
            | OpCode::Add
            | OpCode::Sub
            | OpCode::Mul
            | OpCode::Div
            | OpCode::Neg
            | OpCode::Print => self.code.push(op_code.into()),
            OpCode::Constant(v) => {
                self.values.push(v);
                self.code.push(op_code.into());
                let pos = (self.values.len() - 1) as u8;
                self.code.push(pos);
            }
        }
        self.lines.push(line);
    }
}
impl From<ChunkBuilder> for Chunk {
    fn from(value: ChunkBuilder) -> Self {
        Self {
            code: value.code.into_boxed_slice(),
            lines: value.lines.into_boxed_slice(),
            values: value.values.into_boxed_slice(),
        }
    }
}
