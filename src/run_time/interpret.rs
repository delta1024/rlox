use std::ops::ControlFlow;

use crate::{
    byte_code::OpCode,
    vm::{BinaryOp, UnaryOp, Vm, VmResult}, frame::CallFrame,
};

pub(crate) fn interpret_instruction<'a>(vm: &mut Vm, frame: &CallFrame<'a>, op_code: OpCode) -> ControlFlow<VmResult<()>> {
    
    match op_code {
        OpCode::Constant(v) => vm.push(v),
        OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div => {
            let (Some(b), Some(a)) = (vm.pop(), vm.pop()) else {
		unreachable!()
	    };
            let v = match vm.binary_instruction(BinaryOp::new(op_code, a, b)) {
                Ok(v) => v,
                Err(e) => return ControlFlow::Break(Err(e)),
            };
            vm.push(v);
        }
        OpCode::Neg => {
            let v = vm.pop().unwrap();
            let v = match vm.unary_instruction(UnaryOp::new(op_code, v)) {
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
