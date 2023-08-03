mod byte_code;
mod compiler;
mod frame;
mod heap;
mod heap_objects;
mod interpret;
mod lexer;
mod objects;
mod stack;
mod value;
mod vm;

use compiler::compile;
use lexer::Lexer;

use crate::{
    byte_code::{ChunkBuilder, OpCode},
    frame::CallFrame,
    heap::Heap,
    heap_objects::ObjFunction,
    interpret::interpret_instruction,
    objects::ObjRef,
    stack::CallStack,
    value::Value,
    vm::Vm,
};

fn call_function(
    _vm: &mut Vm,
    call_stack: &mut CallStack,
    function: ObjRef,
) -> Result<(), Box<dyn std::error::Error>> {
    let frame = CallFrame::new(function);
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
    let chunk = match compile("-(1 - 2 + 3)").0 {
        Ok(c) => c,
        Err(err) => panic!("{err}"),
    };
    let mut heap = Heap::new();
    let main_str = heap.allocate_string("_main");
    let main_fn = heap.allocate_object::<ObjFunction>(ObjFunction::new(main_str, chunk));
    let mut vm = Vm::new();
    let mut call_stack = CallStack::new();
    call_function(&mut vm, &mut call_stack, main_fn)?;
    main_loop(&mut vm, &mut call_stack)
}
