mod byte_code;
mod value {
    use std::fmt::Display;
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    #[allow(dead_code)]
    pub(crate) enum Value {
        #[default]
        Nil,
        Int(i64),
        Bool(bool),
    }

    impl Display for Value {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            use Value::*;
            match self {
                Nil => write!(f, "nil"),
                Int(v) => write!(f, "{v}"),
                Bool(v) => write!(f, "{v}"),
            }
        }
    }
}

mod frame;
mod interpret;
mod stack;
mod vm;
use crate::{
    byte_code::{Chunk, ChunkBuilder, OpCode},
    frame::CallFrame,
    interpret::interpret_instruction,
    stack::CallStack,
    value::Value,
    vm::Vm,
};

fn call_function(
    _vm: &mut Vm,
    call_stack: &mut CallStack,
    chunk: Chunk,
) -> Result<(), Box<dyn std::error::Error>> {
    let frame = CallFrame::new(chunk);
    call_stack.push(frame)?;
    Ok(())
}
fn main_loop(vm: &mut Vm, call_stack: &mut CallStack) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let call_pos = call_stack.len();
        if call_pos == 0 {
            break;
        }
        let call_pos = call_pos - 1;
        let instruction = call_stack[call_pos].read_instruction()?;
        interpret_instruction(vm, call_stack, instruction)?;
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut chunk = ChunkBuilder::new();
    chunk.write_byte(OpCode::Constant(Value::Int(32)), 1);
    chunk.write_byte(OpCode::Constant(Value::Int(32)), 1);
    chunk.write_byte(OpCode::Add, 2);
    chunk.write_byte(OpCode::Print, 1);
    chunk.write_byte(OpCode::Return, 2);
    let chunk = Chunk::from(chunk);
    let mut vm = Vm::new();
    let mut call_stack = CallStack::new();
    call_function(&mut vm, &mut call_stack, chunk)?;
    main_loop(&mut vm, &mut call_stack)
}
