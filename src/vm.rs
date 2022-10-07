use std::{fmt, pin::Pin, result};
use crate::{chunk::{self, OpCode, Ip}, value::Value};

static mut VM: Vm = Vm::new();

pub type Result<T> = result::Result<T, Error>;
type Chunk = Pin<Box<chunk::Chunk>>;
pub struct Vm {
    chunk:  Option<Chunk>,
    ip: Option<Ip>,
}

impl Vm {
    const fn new() -> Vm {
        Vm {
            chunk: None,
            ip: None,
        }
    }
    
    pub fn interpret(chunk: chunk::Chunk) -> Result<()> {
        unsafe {
            let chunk = VM.chunk.insert(chunk.pin());
            let _ = VM.ip.insert(chunk.as_ref().ip());
        }
        Self::run()
    }
    
    fn read_byte() -> u8 {
        let ip = unsafe {VM.ip.as_mut().unwrap()};
        ip.next().unwrap()
    }
    
    fn read_constant() -> Value {
        let n = Vm::read_byte();
        let ip = unsafe {
            VM.ip.as_mut().unwrap()
        };
        
        unsafe {
            ip.get_constant(n)
        }
    }

    fn disassemble_instruction() {
        let instruction = unsafe {
            let ip = VM.ip.as_mut().unwrap();
            ip.disassemble_instruction()
        };
        
        println!("{}", instruction);
    }
    fn run() -> Result<()>{
        loop {
            let instruction = Vm::read_byte();

            #[cfg(feature = "trace_execution")]
            Vm::disassemble_instruction();

            match instruction.into() {
                
                OpCode::Return => {
                    return Ok(())
                }
                OpCode::Constant => {
                    let constant = Vm::read_constant();
                    println!("{}", constant);
                }
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    Compile,
    Runtime,
}

impl fmt::Display for Error{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}