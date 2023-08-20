use crate::{frame::pc::PositionCounter, heap::Object, value::Value};
pub(crate) mod lines;
pub(crate) use lines::*;
pub(crate) mod op_code;
use op_code::OP_CODE_MAX;
pub(crate) use op_code::*;
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
            | OpCode::Not
            | OpCode::Equal
            | OpCode::Greater
            | OpCode::Less
            | OpCode::Print
            | OpCode::Pop => {
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
            OpCode::DefineGlobal(v) | OpCode::GetGlobal(v) => {
                self.values.push(Value::Object(Object::from_ptr(&v)));
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
            0 | 2..=OP_CODE_MAX => (n.into(), 1.into()),
            1 => {
                let p = self.code[*pos + 1] as usize;
                let v = self.values[p];
                (OpCode::Constant(v), 2.into())
            }
            16 => {
                let p = self.code[*pos + 1] as usize;
                let Value::Object(v) = self.values[p] else {
		    unreachable!()
		};
                (OpCode::DefineGlobal(v.as_obj()), 2.into())
            }
            17 => {
                let p = self.code[*pos + 1] as usize;
                let Value::Object(v) = self.values[p] else {
		    unreachable!()
		};
                (OpCode::GetGlobal(v.as_obj()), 2.into())
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
