pub mod error;
pub(crate) mod vm;
pub(crate) mod interpret;
pub use error::*;
pub(crate) use interpret::*;
use vm::Vm;
use crate::{
    byte_code::OpCode,
    frame::{pc::PositionCounter, CallFrame},

};
pub(crate) struct RuntimeState<'a, 'b> {
    position: PositionCounter,
    vm: &'a mut Vm,
    frames: &'a mut CallFrame<'b>,
}

impl<'a, 'b> RuntimeState<'a, 'b> {
    pub(crate) fn new(vm: &'a mut Vm, frames: &'a mut CallFrame<'b>) -> Self {
        Self {
            position: Default::default(),
            vm,
            frames,
        }
    }
    #[inline(always)]
    pub(crate) fn get_position(&mut self) -> PositionCounter {
        self.position
    }
    #[inline(always)]
    pub(crate) fn get_vm(&mut self) -> &mut Vm {
        self.vm
    }
    #[inline(always)]
    pub(crate) fn get_frames(&mut self) -> &mut CallFrame<'b> {
        self.frames
    }
    pub(crate) fn advance_position(&mut self) -> OpCode {
        self.position = self.frames.position_conunter;
        self.frames.advance_position()
    }
}
