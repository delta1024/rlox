use crate::{frame::pc::PositionCounter, value::Value};
pub(crate) mod lines;
pub(crate) use lines::*;
#[derive(Debug, Copy, Clone)]
pub(crate) enum OpCode {
    Return,
    Constant(Value),
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    Nil,
    True,
    False,
    Not,
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Return,
            1 => Self::Constant(Value::default()),
            2 => Self::Add,
            3 => Self::Sub,
            4 => Self::Mul,
            5 => Self::Div,
            6 => Self::Neg,
            7 => Self::Nil,
            8 => Self::True,
            9 => Self::False,
	    10 => Self::Not,
            _ => unreachable!(),
        }
    }
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
            OpCode::Nil => 7,
            OpCode::True => 8,
            OpCode::False => 9,
	    OpCode::Not => 10,
        }
    }
}

pub(crate) struct ChunkBuilder {
    code: Vec<u8>,
    values: Vec<Value>,
    lines: LinesBuilder,
}

impl ChunkBuilder {
    pub(crate) fn new() -> Self {
        Self {
            code: Vec::new(),
            values: Vec::new(),
            lines: LinesBuilder::new(),
        }
    }
    pub(crate) fn write_byte(mut self, byte: OpCode, line: usize) -> Self {
        match byte {
            OpCode::Return
            | OpCode::Add
            | OpCode::Sub
            | OpCode::Mul
            | OpCode::Div
            | OpCode::True
            | OpCode::False
            | OpCode::Nil
		| OpCode::Neg
		| OpCode::Not => {
                self.code.push(byte.into());
                self.lines.push(line as u8);
            }
            OpCode::Constant(c) => {
                self.values.push(c);
                let pos = self.values.len() as u8 - 1;
                self.code.push(byte.into());
                self.lines.push(line as u8);
                self.code.push(pos);
                self.lines.push(line as u8);
            }
        }
        self
    }
}
#[derive(Debug)]
pub(crate) struct Chunk {
    code: Box<[u8]>,
    values: Box<[Value]>,
    lines: Lines,
}

impl Chunk {
    pub(crate) fn get_instruction(&self, pos: PositionCounter) -> (OpCode, PositionCounter) {
        let n = self.code[*pos];
        match n {
            0 | 2..=10 => (n.into(), 1.into()),
            1 => {
                let p = self.code[*pos + 1] as usize;
                let v = self.values[p];
                (OpCode::Constant(v), 2.into())
            }
            _ => unreachable!(),
        }
    }
    pub(crate) fn get_line(&self, pos: PositionCounter) -> Option<u8> {
        self.lines.get(pos)
    }
}
impl From<ChunkBuilder> for Chunk {
    fn from(value: ChunkBuilder) -> Self {
        Self {
            code: value.code.into_boxed_slice(),
            values: value.values.into_boxed_slice(),
            lines: value.lines.finalize(),
        }
    }
}

impl FromIterator<(OpCode, usize)> for Chunk {
    fn from_iter<T: IntoIterator<Item = (OpCode, usize)>>(iter: T) -> Self {
        iter.into_iter()
            .fold(
                ChunkBuilder::new(),
                |builder: ChunkBuilder, (code, line)| builder.write_byte(code, line),
            )
            .into()
    }
}
