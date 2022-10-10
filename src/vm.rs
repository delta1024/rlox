pub use crate::error::VmError as Error;
use crate::{
    chunk::{self, Ip, OpCode},
    compiler::compile,
    value::Value,
};
use std::{pin::Pin, ptr, result};

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
            stack: [Value::Null; STACK_MAX],
            stack_top: ptr::null_mut(),
        }
    }

    pub fn init_vm() {
        Vm::reset_stack();
    }

    pub fn push<T: Into<Value>>(value: T) {
        unsafe {
            *VM.stack_top = value.into();
            VM.stack_top = VM.stack_top.add(1);
        }
    }

    pub fn pop() -> Value {
        unsafe {
            VM.stack_top = VM.stack_top.sub(1);
            VM.stack_top.read()
        }
    }
    pub fn peek(distance: isize) -> Value {
        unsafe { VM.stack_top.offset(-1 - distance).read() }
    }
    pub fn reset_stack() {
        unsafe {
            VM.stack_top = VM.stack.as_mut_ptr();
        }
    }

    fn runtime_error(message: &str) -> Result<()> {
        let mut error = String::new();

        error.push_str(message);
        unsafe {
            let ip = VM.ip.as_ref().unwrap();
            let instruction = ip.offset();
            let line = ip.get_lines().get_line(instruction).unwrap();
            let temp = format!("[line {}] in script", line);

            error.push_str(&temp);
        }
        Err(Error::Runtime(error))
    }

    pub fn interpret(source: &str) -> Result<()> {
        let chunk = compile(source)?;
        unsafe {
            let n = VM.chunk.insert(chunk.pin()).as_ref();
            let _ = VM.ip.insert(n.ip());
        }
        Vm::run()
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
        if f64::try_from(a).is_err() || f64::try_from(b).is_err() {
            return Vm::runtime_error("Operands must be numbers.");
        }
        Vm::push(match operator {
            OpCode::Add => a + b,
            OpCode::Subtract => a - b,
            OpCode::Multiply => a * b,
            OpCode::Divide => a / b,
            OpCode::Greater => {
                Vm::push(a > b);
                return Ok(());
            },
            OpCode::Less => {
                Vm::push(a < b);
                return Ok(());
            }
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
                OpCode::True => Vm::push(true),
                OpCode::False => Vm::push(false),
                OpCode::Equal => {
                    let b = Vm::pop();
                    let a = Vm::pop();
                    Vm::push(a == b);
                }
                OpCode::Nil => Vm::push(Value::Null),
                OpCode::Not => {
                    let val = Vm::pop();
                    Vm::push(val.is_falsey());
                }
                OpCode::Negate => {
                    let n = Vm::peek(0);
                    if f64::try_from(n).is_err() {
                        return Vm::runtime_error("Operand must be a number.");
                    }

                    Vm::push(-Vm::pop());
                }
                OpCode::Add | OpCode::Subtract | OpCode::Divide | OpCode::Multiply | OpCode::Greater | OpCode::Less => {
                    Vm::binary_op(instruction.into())?
                }
            }
        }
    }
}
