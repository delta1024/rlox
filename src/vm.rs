pub use crate::error::VmError as Error;
use crate::{
    chunk::{self, Ip, OpCode},
    compiler::compile,
    objects::{Obj, ObjFunction, ObjString, ObjType},
    value::Value,
};
use std::{
    collections::{HashMap, LinkedList},
    pin::Pin,
    ptr, result,
};

pub static mut VM: Vm = Vm::new();

pub type Result<T> = result::Result<T, Error>;
type Chunk = Pin<Box<chunk::Chunk>>;
const FRAMES_MAX: usize = 64;
const STACK_MAX: usize = FRAMES_MAX * u8::MAX as usize;

#[derive(Clone, Copy)]
struct CallFrame {
    function: *const ObjFunction,
    ip: Ip,
    slots: *mut Value,
}
impl CallFrame {
    const fn new() -> CallFrame {
        CallFrame {
            function: ptr::null(),
            ip: Ip::null(),
            slots: ptr::null_mut(),
        }
    }
}
pub struct Vm {
    frames: [CallFrame; FRAMES_MAX],
    frame_count: usize,
    stack: [Value; STACK_MAX],
    stack_top: *mut Value,
    _objects: LinkedList<Pin<Box<dyn Obj>>>,
    pub strings: Option<HashMap<String, Pin<Box<ObjString>>>>,
    pub globals: Option<HashMap<String, Value>>,
}

impl Vm {
    const fn new() -> Vm {
        Vm {
            frames: [CallFrame::new(); FRAMES_MAX],
            frame_count: 0,
            stack: [Value::Null; STACK_MAX],
            stack_top: ptr::null_mut(),
            _objects: LinkedList::new(),
            strings: None,
            globals: None,
        }
    }

    pub fn init_vm() {
        Vm::reset_stack();
        unsafe {
            let _ = VM.strings.insert(HashMap::new());
            _ = VM.globals.insert(HashMap::new());
        }
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
            VM.frame_count = 0;
        }
    }
    pub fn allocate_obj(obj: Pin<Box<dyn Obj>>) -> *mut dyn Obj {
        unsafe {
            VM._objects.push_back(obj);
            let n = VM._objects.back_mut().unwrap().as_mut();
            let n: *mut dyn Obj = Pin::get_mut(n);
            n
        }
    }
    fn runtime_error(message: &str) -> Result<()> {
        let mut error = String::new();

        error.push_str(message);
        error.push('\n');
        unsafe {
            let frame = &VM.frames[VM.frame_count - 1];

            let ip = frame.ip;
            let instruction = ip.offset();
            let line = ip.get_lines().get_line(instruction).unwrap();
            let temp = format!("[line {}] in script", line);

            error.push_str(&temp);
        }
        Err(Error::Runtime(error))
    }

    pub fn interpret(source: &str) -> Result<()> {
        let function = compile(source)?;
        unsafe {
            let function: *const dyn Obj = function;
            Vm::push(function);
        }
        let frame = unsafe {
            VM.frame_count += 1;
            &mut VM.frames[VM.frame_count - 1]
        };
        frame.function = function;
        frame.ip = unsafe {
            let function = function.as_ref().unwrap();
            function.chunk.ip()
        };
        frame.slots = unsafe { &mut VM.stack[0] };
        Vm::run()
    }

    fn read_byte(frame: *mut CallFrame) -> u8 {
        let ip = unsafe { &mut frame.as_mut().unwrap().ip };
        ip.next().unwrap()
    }

    fn read_constant(frame: *mut CallFrame) -> Value {
        let n = Vm::read_byte(frame);
        let ip = unsafe { frame.as_mut().unwrap().ip };

        unsafe { ip.get_constant(n) }
    }

    fn read_short(frame: *mut CallFrame) -> usize {
        let ip = unsafe { &mut frame.as_mut().unwrap().ip };
        ip.next();
        ip.next();
        let (a, b) = ip.short_bytes();
        let (a, b) = (a as u16, b as u16);
        let n = (a << 8) | b;
        n as usize
    }
    #[cfg(feature = "trace_execution")]
    fn disassemble_instruction(frame: *const CallFrame) {
        let instruction = unsafe {
            let ip = frame.as_ref().unwrap().ip;
            ip.disassemble_instruction()
        };

        println!("{}", instruction.0);
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
            }
            OpCode::Less => {
                Vm::push(a < b);
                return Ok(());
            }
            _ => unreachable!(),
        });
        Ok(())
    }
    fn concatenate() {
        let b = Vm::pop();
        let b = b.as_obj().unwrap().as_rstring();
        let a = Vm::pop();
        let a = a.as_obj().unwrap().as_rstring();
        let c = ObjString::concat(a, b);
        let c = crate::objects::allocate_string(c.as_rstring());
        Vm::push(c);
    }
    fn run() -> Result<()> {
        let frame: *mut CallFrame = unsafe { &mut VM.frames[VM.frame_count - 1] };
        loop {
            let instruction = Vm::read_byte(frame);

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
                Vm::disassemble_instruction(frame);
            }

            match instruction.into() {
                OpCode::Return => {
                    Vm::pop();
                    return Ok(());
                }
                OpCode::Print => {
                    println!("{}", Vm::pop());
                }
                OpCode::Constant => {
                    let constant = Vm::read_constant(frame);
                    Vm::push(constant);
                }
                OpCode::True => Vm::push(true),
                OpCode::False => Vm::push(false),
                OpCode::Pop => {
                    Vm::pop();
                }
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
                OpCode::Add => {
                    let a = Vm::peek(0);
                    let b = Vm::peek(1);
                    let is_obj = {
                        let n = a.as_obj().is_ok();
                        let m = b.as_obj().is_ok();
                        (n, m)
                    };

                    if is_obj.0 == true && is_obj.1 == true {
                        let a = a.as_obj().unwrap().id();
                        let b = b.as_obj().unwrap().id();
                        if a == ObjType::String && b == ObjType::String {
                            Vm::concatenate();
                        }
                    } else if f64::try_from(a).is_ok() && f64::try_from(b).is_ok() {
                        Vm::binary_op(instruction.into())?;
                    } else {
                        return Vm::runtime_error("Operands must be two strings or two numbers.");
                    }
                }
                OpCode::DefineGlobal => {
                    let name = Vm::read_constant(frame);
                    let name = name.as_obj().unwrap().as_rstring();
                    let table = unsafe { VM.globals.as_mut().unwrap() };
                    table.insert(name.to_owned(), Vm::peek(0));
                    Vm::pop();
                }
                OpCode::GetGlobal => {
                    let name = Vm::read_constant(frame);
                    let name = name.as_obj().unwrap().as_rstring();
                    if let Some(value) = unsafe { VM.globals.as_mut().unwrap().get(name) } {
                        Vm::push(*value);
                    } else {
                        return Vm::runtime_error(&format!("Undefine variable '{}'.", name));
                    }
                }
                OpCode::SetGlobal => {
                    let name = Vm::read_constant(frame);
                    let name = name.as_obj().unwrap().as_rstring();
                    let table = unsafe { VM.globals.as_mut().unwrap() };
                    if let None = table.insert(name.to_owned(), Vm::peek(0)) {
                        table.remove(name);
                        return Vm::runtime_error(&format!("Undefined variable '{}'", name));
                    }
                }
                OpCode::GetLocal => {
                    let slot = Vm::read_byte(frame) as usize;
                    let frame = unsafe { frame.as_mut().unwrap() };
                    let val = unsafe { frame.slots.add(slot).read() };
                    Vm::push(val);
                }
                OpCode::SetLocal => {
                    let slot = Vm::read_byte(frame) as usize;
                    let val = Vm::peek(0);
                    unsafe {
                        let frame = frame.as_mut().unwrap();
                        frame.slots.add(slot).write(val);
                    }
                }
                OpCode::Jump => {
                    let offset = Vm::read_short(frame);
                    unsafe {
                        let frame = frame.as_mut().unwrap();
                        frame.ip.jump_forward(offset);
                    }
                }
                OpCode::JumpIfFalse => {
                    let offset = Vm::read_short(frame);
                    if Vm::peek(0).is_falsey() {
                        unsafe {
                            let frame = frame.as_mut().unwrap();
                            frame.ip.jump_forward(offset);
                        }
                    }
                }
                OpCode::Loop => {
                    let offset = Vm::read_short(frame);
                    unsafe {
                        let frame = frame.as_mut().unwrap();
                        frame.ip.jump_back(offset);
                    }
                }
                OpCode::Subtract
                | OpCode::Divide
                | OpCode::Multiply
                | OpCode::Greater
                | OpCode::Less => Vm::binary_op(instruction.into())?,
            }
        }
    }
}
