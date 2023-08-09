use std::ops::ControlFlow;

use byte_code::{Chunk, OpCode};
use compiler::{CompilerError, Parser};
use frame::CallFrame;
use lexer::Lexer;
use vm::{VmError, Vm};
mod byte_code;
mod stack;
mod compiler;
mod frame;
mod lexer;
mod vm;
mod run_time;

mod value {
    pub(crate) type Value = i64;
}

fn interpret_loop<'a>(vm: &mut Vm,  call_frame:&mut CallFrame<'a>) -> Result<(), VmError> {
    loop {
	let op = call_frame.advance_position();
	match run_time::interpret_instruction(vm, call_frame, op){
	    ControlFlow::Break(r) => break r,
	    ControlFlow::Continue(()) => continue,
	}
	
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chunk = match Parser::new(Lexer::new("1 + 2")).collect::<Result<Chunk, CompilerError>>() {
	Ok(c) => c,
	Err(err) => {
	    eprintln!("{err}");
	    std::process::exit(1);
	}
    };
    let mut vm = Vm::new();
    let mut frame = CallFrame::new(&chunk);
    match    interpret_loop(&mut vm, &mut frame) {
	Ok(()) => Ok(()),
	Err(err) => {
	    eprintln!("{err}");
	    std::process::exit(1);
	}
    }


}
