use std::ops::ControlFlow;

use byte_code::Chunk;
use compiler::{CompilerError, Parser};
use frame::CallFrame;
use heap::{Heap, ObjString};
use run_time::{RuntimeError, RuntimeState};
mod byte_code;
mod compiler;
mod frame;
mod lexer;
mod run_time;
mod stack;
mod heap;
use run_time::vm::Vm;

mod value;

fn main_loop<'a>(vm: &mut Vm, call_frame: &mut CallFrame<'a>) -> Result<(), RuntimeError> {
    let mut state = RuntimeState::new(vm, call_frame);
    loop {
        let op = state.advance_position();
        match run_time::interpret_instruction(&mut state, op) {
            ControlFlow::Break(r) => break r,
            ControlFlow::Continue(()) => continue,
        }
    }
}
fn main() {
    let mut heap = Heap::new();
    let chunk = match Parser::new("\"hello\"", heap.allocator()).collect::<Result<Chunk, CompilerError>>() {
        Ok(c) => c,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };
    let mut vm = Vm::new();
    let mut frame = CallFrame::new(&chunk);
    if let Err(err) = main_loop(&mut vm, &mut frame) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
