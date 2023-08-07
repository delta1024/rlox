use std::ops::ControlFlow;

use crate::{
    byte_code::OpCode,
    vm::{BinaryOp, UnaryOp, Vm, VmResult},
};

pub(crate) fn interpret<'a>(vm: &mut Vm, value: OpCode) -> ControlFlow<VmResult<()>> {
    match value {
        OpCode::Constant(v) => vm.push(v),
        OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div => {
            let (Some(b), Some(a)) = (vm.pop(), vm.pop()) else {
		unreachable!()
	    };
            let v = match vm.binary_instruction(BinaryOp::new(value, a, b)) {
                Ok(v) => v,
                Err(e) => return ControlFlow::Break(Err(e)),
            };
            vm.push(v);
        }
        OpCode::Neg => {
            let v = vm.pop().unwrap();
            let v = match vm.unary_instruction(UnaryOp::new(value, v)) {
                Ok(v) => v,
                Err(err) => return ControlFlow::Break(Err(err)),
            };
            vm.push(v);
        }
        OpCode::Return => {
            println!("{}", vm.pop().unwrap());
            return ControlFlow::Break(Ok(()));
        }
    }
    ControlFlow::Continue(())
}
