use std::ops::ControlFlow;

use super::vm::{BinaryOp, UnaryOp, Vm, VmResult};
use super::RuntimeState;
use crate::byte_code::OpCode;
use crate::value::Value;

pub(crate) fn interpret_instruction<'a, 'b>(
    state: &mut RuntimeState<'b, 'a>,
    op_code: OpCode,
) -> ControlFlow<VmResult<()>> {
    match op_code {
        OpCode::Constant(v) => state.get_vm().push(v),
        OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div => {
            let (b, a) = (
                *state.get_vm().stack.peek(1).unwrap(),
                *state.get_vm().stack.peek(0).unwrap(),
            );
            let v = match Vm::binary_instruction(state, BinaryOp::new(op_code, a, b)) {
                Ok(v) => {
                    (0..=1).for_each(|_| {
                        _ = state.get_vm().stack.pop();
                    });
                    v
                }
                Err(e) => return ControlFlow::Break(Err(e)),
            };

            state.get_vm().push(v);
        }
        OpCode::Neg | OpCode::Not => {
            let v = *state.get_vm().stack.peek(0).unwrap();
            let v = match Vm::unary_instruction(state, UnaryOp::new(op_code, v)) {
                Ok(v) => {
                    _ = state.get_vm().stack.pop();
                    v
                }
                Err(err) => return ControlFlow::Break(Err(err)),
            };
            state.get_vm().push(v);
        }
        OpCode::Nil => state.get_vm().push(Value::Nil),
        OpCode::True => state.get_vm().push(true.into()),
        OpCode::False => state.get_vm().push(false.into()),
        OpCode::Return => {
            println!("{}", state.get_vm().pop().unwrap());
            return ControlFlow::Break(Ok(()));
        }
    }
    ControlFlow::Continue(())
}
