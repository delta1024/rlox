use std::fmt::Display;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum OpCode {
    Constant(Value),
    Return,
}
#[repr(transparent)]
#[derive(Default, Debug)]
struct PositionCounter(pub u16);
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Value {
    #[default]
    Nil,
    Int(i64),
    Bool(bool),
}
#[derive(Debug, Clone,Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ReadInstructionError {
    InvaildInstruction,
}
impl std::error::Error for ReadInstructionError {}
impl Display for ReadInstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	write!(f,"Invalid opcode")
    }
}
struct Chunk {
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
            0 => Ok((Return, position + 1)),
            1 => Ok((Constant(self.values[self.code[position + 1] as usize]), position + 2)),
            _ => Err(ReadInstructionError::InvaildInstruction),
        }
    }
} 
#[derive(Default, Debug)]
struct ChunkBuilder {
    code: Vec<u8>,
    values: Vec<Value>,
    lines: Vec<u8>,
}
impl ChunkBuilder {
    fn new() -> Self {
	Self::default()
    }
    fn write_byte(&mut self, op_code: OpCode, line: u8) {
	match op_code {
	    OpCode::Return => self.code.push(0),
	    OpCode::Constant(v) => {
		self.values.push(v);
		self.code.push(1);
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
fn main() {
    let pos = PositionCounter(0);
    let mut chunk = ChunkBuilder::new();
    chunk.write_byte(OpCode::Constant(Value::Int(32)), 1);
    chunk.write_byte(OpCode::Return, 2);
    let chunk = Chunk::from(chunk);
    let (op_code, new_pos) = match chunk.read_instruction(pos) {
	Ok(v) => v,
	Err(err) => panic!("{err}")
    };
    let pos = PositionCounter(new_pos as u16);
    println!("op_code: {op_code:?}\npos: {pos:?}, ")
}
