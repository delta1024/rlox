use std::{
    fs::File,
    io::{self, Read, Write},
    ops::ControlFlow,
    process::exit,
};

mod byte_code;
mod compiler;
mod error;
mod frame;
mod heap;
mod lexer;
mod run_time;
mod stack;
mod value;

use byte_code::Chunk;
use compiler::{CompilerError, Parser};
use error::Error;
use frame::CallFrame;
use heap::Heap;
use run_time::{vm::Vm, RuntimeError, RuntimeState};

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

fn run_repl() -> Result<(), Error> {
    let mut buffer = String::new();
    let mut heap = Heap::new();
    let mut vm = Vm::new(heap.allocator());
    loop {
        buffer.clear();
        print!("> ");
        io::stdout().flush()?;
        if 0 == io::stdin().read_line(&mut buffer)? {
            break Ok(());
        }
        let chunk = match Parser::new(&buffer, heap.allocator())
            .collect::<Result<Chunk, CompilerError>>()
        {
            Ok(c) => c,
            Err(err) => {
                eprintln!("{err}");
                continue;
            }
        };
        let mut frame = CallFrame::new(&chunk);
        main_loop(&mut vm, &mut frame)?;
    }
}
fn run_file(file_name: &str) -> Result<(), Error> {
    let mut file = File::open(file_name)?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;

    let mut heap = Heap::new();
    let mut vm = Vm::new(heap.allocator());
    let chunk =
        Parser::new(&file_contents, heap.allocator()).collect::<Result<Chunk, CompilerError>>()?;
    let mut frame = CallFrame::new(&chunk);
    main_loop(&mut vm, &mut frame).map_err(|e| e.into())
}
fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if let Err(err) = if args.len() > 1 {
        run_file(&args[1])
    } else {
        run_repl()
    } {
        eprintln!("{err}");
        exit(1);
    }
}
