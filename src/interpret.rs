use crate::{byte_code::OpCode, stack::CallStack, vm::Vm};
pub(crate) fn interpret_instruction(
    vm: &mut Vm,
    call_stack: &mut CallStack,
    instruction: OpCode,
) -> Result<(), Box<dyn std::error::Error>> {
    match instruction {
        OpCode::Constant(c) => vm.push(c)?,
        OpCode::Print => {
            if let Some(v) = vm.pop() {
                println!("{v}");
            }
        }
        OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div => {
            let val = vm.binary_operation(instruction.into())?;
            vm.push(val)?;
        }

        OpCode::Return => _ = call_stack.pop(),
    }
    Ok(())
}
