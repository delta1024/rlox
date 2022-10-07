use crate::{
    chunk::{self, Ip, OpCode},
    value::Value,
};
use std::{fmt, pin::Pin, ptr, result};

static mut VM: Vm = Vm::new();

pub type Result<T> = result::Result<T, Error>;
type Chunk = Pin<Box<chunk::Chunk>>;
const STACK_MAX: usize = 256;
pub struct Vm {
    chunk: Option<Chunk>,
    ip: Option<Ip>,
    stack: [Value; STACK_MAX],
    stack_top: *mut Value,
}

impl Vm {
    const fn new() -> Vm {
        Vm {
            chunk: None,
            ip: None,
            stack: [0.0; STACK_MAX],
            stack_top: ptr::null_mut(),
        }
    }
    pub fn init_vm() {
        Vm::reset_stack();
    }

    pub fn push(value: Value) {
        unsafe {
            *VM.stack_top = value;
            VM.stack_top = VM.stack_top.add(1);
        }
    }

    pub fn pop() -> Value {
        unsafe {
            VM.stack_top = VM.stack_top.sub(1);
            VM.stack_top.read()
        }
    }

    pub fn reset_stack() {
        unsafe {
            VM.stack_top = VM.stack.as_mut_ptr();
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
        let ip = unsafe { VM.ip.as_mut().unwrap() };
        ip.next().unwrap()
    }

    fn read_constant() -> Value {
        let n = Vm::read_byte();
        let ip = unsafe { VM.ip.as_mut().unwrap() };

        unsafe { ip.get_constant(n) }
    }

    fn disassemble_instruction() {
        let instruction = unsafe {
            let ip = VM.ip.as_mut().unwrap();
            ip.disassemble_instruction()
        };

        println!("{}", instruction);
    }
    
    fn binary_op(operator: OpCode) -> Result<()> {
        let b = Vm::pop();
        let a = Vm::pop();
        Vm::push(match  operator { 
            OpCode::Add => a + b,
            OpCode::Subtract => a - b,
            OpCode::Multiply => a * b,
            OpCode::Divide => a / b,
            _ => unreachable!(),

        });
        Ok(())
    }
    fn run() -> Result<()> {
        loop {
            let instruction = Vm::read_byte();

            #[cfg(feature = "trace_execution")]
            {
                print!("          ");
                unsafe {
                    let offset = VM.stack_top.offset_from(VM.stack.as_mut_ptr()) as usize;
                    for i in &VM.stack[0..offset] {
                        print!("[ {} ]", i);
                    }
                }
                print!("\n");
                Vm::disassemble_instruction();
            }

            match instruction.into() {
                OpCode::Return => {
                    println!("{}", Vm::pop());
                    return Ok(());
                }
                OpCode::Constant => {
                    let constant = Vm::read_constant();
                    Vm::push(constant);
                }
                OpCode::Negate => Vm::push(-Vm::pop()),
                OpCode::Add | OpCode::Subtract | OpCode::Divide | OpCode::Multiply => Vm::binary_op(instruction.into())?,
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
