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
mod interpret;

mod value {
    pub(crate) type Value = i64;
}

fn interpret_loop<'a>(vm: &mut Vm, mut call_frame: CallFrame<'a>) -> Result<(), VmError> {
    loop {
	match interpret::interpret(vm, call_frame.advance_position()) {
	    ControlFlow::Break(r) => break r,
	    ControlFlow::Continue(()) => continue,
	}
	
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chunk = match Parser::new(Lexer::new("1 +")).collect::<Result<Chunk, CompilerError>>() {
	Ok(c) => c,
	Err(err) => {
	    eprintln!("{err}");
	    std::process::exit(1);
	}
    };
    let mut vm = Vm::new();
    
    match    interpret_loop(&mut vm, CallFrame::new(&chunk)) {
	Ok(()) => Ok(()),
	Err(err) => {
	    eprintln!("{err}");
	    std::process::exit(1);
	}
    }


}
