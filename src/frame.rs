use crate::byte_code::{Chunk, OpCode, ReadInstructionError};
#[repr(transparent)]
#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct PositionCounter(pub u16);
pub(crate) struct CallFrame {
    code: Chunk,
    position_counter: PositionCounter,
}

impl CallFrame {
    pub(crate) fn new(code: Chunk) -> CallFrame {
        CallFrame {
            code,
            position_counter: PositionCounter::default(),
        }
    }

    pub(crate) fn read_instruction(&mut self) -> Result<OpCode, ReadInstructionError> {
        let (instruction, pos) = self.code.read_instruction(self.position_counter)?;
        self.position_counter = PositionCounter(pos as u16);
        Ok(instruction)
    }
}
