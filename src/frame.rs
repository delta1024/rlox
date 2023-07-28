use crate::{
    byte_code::{OpCode, ReadInstructionError},
    heap_objects::ObjFunction,
    objects::ObjRef,
};
#[repr(transparent)]
#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct PositionCounter(pub u16);
pub(crate) struct CallFrame {
    function: ObjRef,
    position_counter: PositionCounter,
}

impl CallFrame {
    pub(crate) fn new(code: ObjRef) -> CallFrame {
        CallFrame {
            function: code,
            position_counter: PositionCounter::default(),
        }
    }

    pub(crate) fn read_instruction(&mut self) -> Result<OpCode, ReadInstructionError> {
        let (instruction, pos) = (self
            .function
            .get_ref::<ObjFunction>()
            .map_err(|_| ReadInstructionError::InvaildInstruction)?)
        .chunk
        .read_instruction(self.position_counter)?;
        self.position_counter = PositionCounter(pos as u16);
        Ok(instruction)
    }
}
