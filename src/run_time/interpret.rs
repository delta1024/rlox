use std::ops::ControlFlow;

use crate::{
    byte_code::OpCode,
    frame::CallFrame,
    vm::{BinaryOp, UnaryOp, Vm, VmResult},
};

use super::RuntimeState;

pub(crate) fn interpret_instruction<'a, 'b>(
    state: &mut RuntimeState<'b, 'a>,
    op_code: OpCode,
) -> ControlFlow<VmResult<()>> {
    match op_code {
        OpCode::Constant(v) => state.get_vm().push(v),
        OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div => {
            let (Some(b), Some(a)) = (state.get_vm().pop(), state.get_vm().pop()) else {
		unreachable!()
	    };
            let v = match state
                .get_vm()
                .binary_instruction(BinaryOp::new(op_code, a, b))
            {
                Ok(v) => v,
                Err(e) => return ControlFlow::Break(Err(e)),
            };
            state.get_vm().push(v);
        }
        OpCode::Neg => {
            let v = state.get_vm().pop().unwrap();
            let v = match state.get_vm().unary_instruction(UnaryOp::new(op_code, v)) {
                Ok(v) => v,
                Err(err) => return ControlFlow::Break(Err(err)),
            };
            state.get_vm().push(v);
        }
        OpCode::Return => {
            println!("{}", state.get_vm().pop().unwrap());
            return ControlFlow::Break(Ok(()));
        }
    }
    ControlFlow::Continue(())
}
