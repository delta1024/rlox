pub use crate::error::VmError as Error;
use crate::{
    chunk::{Ip, OpCode},
    compiler::compile,
    objects::{
        allocate_string, NativeFn, Obj, ObjBoundMethod, ObjClass, ObjClosure, ObjInstance,
        ObjNative, ObjString, ObjType, ObjUpvalue,
    },
    value::Value,
};
#[cfg(feature = "log_gc")]
use std::alloc::Layout;
use std::{
    collections::{HashMap, LinkedList},
    pin::Pin,
    ptr, result,
    time::SystemTime,
};
const GC_START_THRESHOLD: usize = 1024 * 1024;
const FRAMES_MAX: usize = 64;
const STACK_MAX: usize = FRAMES_MAX * u8::MAX as usize;
pub type Result<T> = result::Result<T, Error>;

pub static mut VM: Vm = Vm {
    frames: [CallFrame::new(); FRAMES_MAX],
    frame_count: 0,
    stack: [Value::Null; STACK_MAX],
    stack_top: ptr::null_mut(),
    objects: Some(LinkedList::new()),
    strings: None,
    globals: None,
    open_upvalues: ptr::null_mut(),
    gray_stack: None,
    bytes_allocated: 0,
    next_gc: GC_START_THRESHOLD,
    init_string: ptr::null_mut(),
};

fn clock_native(_: &[Value]) -> Value {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
        .into()
}

#[derive(Clone, Copy)]
pub struct CallFrame {
    pub closure: *mut ObjClosure,
    pub ip: Ip,
    pub slots: *mut Value,
}
impl CallFrame {
    const fn new() -> CallFrame {
        CallFrame {
            closure: ptr::null_mut(),
            ip: Ip::null(),
            slots: ptr::null_mut(),
        }
    }

    pub fn slots(&self) -> &mut [Value] {
        let offset = unsafe { self.slots.offset_from(&VM.stack[0]) };
        let remaining = STACK_MAX - offset as usize;
        unsafe { std::slice::from_raw_parts_mut(self.slots, remaining) }
    }
    pub fn closure(&self) -> &mut ObjClosure {
        unsafe {
            self.closure
                .as_mut()
                .expect("Uninitialized closure in call frame.")
        }
    }
}
pub struct Vm {
    pub frames: [CallFrame; FRAMES_MAX],
    pub frame_count: usize,
    pub stack: [Value; STACK_MAX],
    pub stack_top: *mut Value,
    pub gray_stack: Option<Vec<&'static mut dyn Obj>>,
    pub objects: Option<LinkedList<Pin<Box<dyn Obj>>>>,
    pub strings: Option<HashMap<String, Pin<Box<ObjString>>>>,
    pub globals: Option<HashMap<*mut ObjString, Value>>,
    pub bytes_allocated: usize,
    pub next_gc: usize,
    pub init_string: *mut ObjString,
    pub open_upvalues: *mut ObjUpvalue,
}

impl Vm {
    pub fn free_vm() {
        unsafe {
            _ = VM.objects.take();
            _ = VM.strings.take();
            _ = VM.globals.take();
            #[cfg(feature = "log_gc")]
            println!("bytes allocated: {}", VM.bytes_allocated);
        }
    }
    pub fn init_vm() {
        Vm::reset_stack();
        unsafe {
            let _ = VM.strings.insert(HashMap::new());
            _ = VM.globals.insert(HashMap::new());
            VM.init_string = allocate_string("init");
        }
        Vm::define_native("clock", clock_native);
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
    fn call(closure: &mut ObjClosure, arg_count: u32) -> Result<()> {
        if arg_count != closure.function().arity {
            Vm::runtime_error(&format!(
                "Expected {} arguments but got {}",
                closure.function().arity,
                arg_count
            ))?;
        }

        if unsafe { VM.frame_count == FRAMES_MAX } {
            Vm::runtime_error("Stack overflow.")?;
        }
        let frame = unsafe { &mut VM.frames[VM.frame_count] };
        unsafe {
            VM.frame_count += 1;
        }
        frame.closure = closure;
        frame.ip = closure.function().chunk.ip();
        frame.slots = unsafe { VM.stack_top.sub(arg_count as usize).sub(1) };

        Ok(())
    }
    pub fn call_value(mut callee: Value, arg_count: u32) -> Result<()> {
        match callee.as_obj_mut() {
            Ok(obj) if obj.id() == ObjType::Class => {
                let klass = obj.as_class_mut().unwrap();
                unsafe {
                    VM.stack_top
                        .sub(arg_count as usize)
                        .sub(1)
                        .write((ObjInstance::new(klass) as *mut dyn Obj).into());
                }
                unsafe {
                    if let Some(initializer) = klass.methods.get(&VM.init_string) {
                        let mut initializer = *initializer;
                        return Vm::call(
                            initializer
                                .as_obj_mut()
                                .expect("Expected object value.")
                                .as_closure_mut()
                                .expect("Expected closure"),
                            arg_count,
                        );
                    } else if arg_count != 0 {
                        return Vm::runtime_error(&format!(
                            "Expect 0 arguments but got {}",
                            arg_count
                        ));
                    };
                }
                Ok(())
            }
            Ok(obj) if obj.id() == ObjType::Closure => {
                let closure = obj.as_closure_mut().expect("Expected closure obj");
                Vm::call(closure, arg_count)
            }
            Ok(obj) if obj.id() == ObjType::Native => {
                let native = obj.as_native().unwrap().function;
                let offset = unsafe { VM.stack_top.offset_from(&VM.stack[0]) as usize };
                let slice = unsafe { &VM.stack[(offset - arg_count as usize)..offset] };
                let result = native(slice);
                unsafe {
                    VM.stack_top = VM.stack_top.sub(arg_count as usize + 1);
                }
                Vm::push(result);
                Ok(())
            }
            Ok(obj) if obj.id() == ObjType::BoundMethod => {
                let bound = obj.as_bound_method_mut().unwrap();
                let method = unsafe { bound.method.as_mut().expect("uninitialized closure.") };
                unsafe {
                    VM.stack_top
                        .sub(arg_count as usize)
                        .sub(1)
                        .write(bound.reciever);
                }
                Vm::call(method, arg_count)
            }
            _ => {
                println!("{}", callee.as_obj().unwrap());
                Vm::runtime_error("Can only call functions and classes.")
            }
        }
    }
    fn invoke_from_class(klass: *mut ObjClass, name: *mut ObjString, arg_count: u8) -> Result<()> {
        let klass = unsafe { klass.as_mut().unwrap() };
        let Some(method) = klass.methods.get(&name) else {
            return Vm::runtime_error(&format!("Undefined property '{}'", unsafe {name.as_ref().unwrap()}));
        };
        let mut method = *method;
        let method = method.as_obj_mut().unwrap().as_closure_mut().unwrap();
        Vm::call(method, arg_count as u32)
    }
    fn invoke(name: *mut ObjString, arg_count: u8) -> Result<()> {
        let mut reciever = Vm::peek(arg_count as isize);
        let Some(instance) = reciever
            .as_obj_mut()
            .expect("Expected object value")
            .as_instance_mut() else {
          return Vm::runtime_error("Only instances have methods.");
        };

        if let Some(value) = instance.fields.get(&name) {
            unsafe {
                VM.stack_top.sub(arg_count as usize).sub(1).write(*value);
                return Vm::call_value(*value, arg_count as u32);
            }
        }

        Vm::invoke_from_class(instance.klass, name, arg_count)
    }
    fn bind_method(klass: *mut ObjClass, name: *mut ObjString) -> Result<()> {
        let klass = unsafe { klass.as_mut().expect("uninitialized class") };
        let Some(method) = klass.methods.get(&name) else {
            return Vm::runtime_error(&format!("Undefined property {}", unsafe {name.as_ref().expect("uninitialized string.")}));
        };
        let mut method = *method;
        let method = method
            .as_obj_mut()
            .expect("Expect object vlaue")
            .as_closure_mut()
            .expect("Expected closure");
        let bound = ObjBoundMethod::new(Vm::peek(0), method);
        Vm::pop();
        Vm::push(bound as *mut dyn Obj);
        Ok(())
    }
    fn capture_upvalue(local: *mut Value) -> *mut ObjUpvalue {
        let mut prev_upvale = ptr::null_mut();
        let mut upvalue = unsafe { VM.open_upvalues };
        while !upvalue.is_null() && {
            let upvalue = unsafe { upvalue.as_ref().unwrap() };
            upvalue.location > local
        } {
            prev_upvale = upvalue;
            upvalue = unsafe { upvalue.as_ref().unwrap().next };
        }

        if !upvalue.is_null() && {
            let upvalue = unsafe { upvalue.as_ref().unwrap() };
            upvalue.location == local
        } {
            return upvalue;
        }
        let created_upvale = ObjUpvalue::new(local);
        unsafe { created_upvale.as_mut() }
            .expect("uninitialized upvalue.")
            .next = upvalue;
        if prev_upvale.is_null() {
            unsafe {
                VM.open_upvalues = created_upvale;
            }
        } else {
            unsafe {
                prev_upvale.as_mut().unwrap().next = created_upvale;
            }
        }
        created_upvale
    }
    pub fn close_upvalue(last: *mut Value) {
        unsafe {
            while !VM.open_upvalues.is_null() && VM.open_upvalues.as_ref().unwrap().location >= last
            {
                let upvalue = VM.open_upvalues.as_mut().unwrap();
                upvalue.closed = *upvalue.location;
                upvalue.location = &mut upvalue.closed;
                VM.open_upvalues = upvalue.next;
            }
        }
    }
    pub fn define_method(name: *mut ObjString) {
        let method = Vm::peek(0);
        let mut klass = Vm::peek(1);
        let klass = klass
            .as_obj_mut()
            .expect("Expected class object.")
            .as_class_mut()
            .expect("Expect class.");
        klass.methods.insert(name, method);
        Vm::pop();
    }
    pub fn reset_stack() {
        unsafe {
            VM.open_upvalues = ptr::null_mut();
            VM.stack_top = VM.stack.as_mut_ptr();
            VM.frame_count = 0;
        }
    }
    pub fn allocate_obj<T: Obj + 'static>(obj: T) -> *mut dyn Obj {
        #[cfg(feature = "log_gc")]
        let layout = Layout::for_value(&obj);
        unsafe {
            VM.objects
                .as_mut()
                .expect("uninitialized vm")
                .push_back(Box::pin(obj));
            let m = VM
                .objects
                .as_mut()
                .expect("uninitialized vm")
                .back_mut()
                .unwrap()
                .as_mut();
            #[cfg(feature = "log_gc")]
            let id = m.id();
            let n: *mut dyn Obj = Pin::get_mut(m);
            #[cfg(feature = "log_gc")]
            println!("{:?} allocate {} for {:?}", n, layout.size(), id);
            n
        }
    }

    fn runtime_error(message: &str) -> Result<()> {
        let mut error = String::new();

        error.push_str(message);
        error.push('\n');

        unsafe {
            for i in (0..=(VM.frame_count - 1)).rev() {
                let frame = &VM.frames[i];
                let function = frame.closure().function;
                let function = function.as_ref().unwrap();
                let instruction = frame.ip.offset() - 1;
                error.push_str(&format!(
                    "[line {}] in ",
                    frame.ip.get_lines().get_line(instruction).unwrap()
                ));
                if function.name.is_null() {
                    error.push_str("script\n");
                } else {
                    error.push_str(&format!("{}()\n", function.as_rstring()));
                }
            }
        }
        eprintln!("{}", error);
        Vm::reset_stack();
        Err(Error::Runtime(error))
    }

    pub fn define_native(name: &str, function: NativeFn) {
        Vm::push(allocate_string(name) as *mut dyn Obj);
        Vm::push(ObjNative::new(function) as *mut dyn Obj);
        unsafe {
            VM.globals.as_mut().unwrap().insert(
                VM.stack[0].as_obj_mut().unwrap().as_string_mut().unwrap(),
                VM.stack[1],
            );
        }
        Vm::pop();
        Vm::pop();
    }
    pub fn interpret(source: &str) -> Result<()> {
        let function = compile(source)?;
        Vm::push(function as *mut dyn Obj);
        let closure = unsafe {
            ObjClosure::new(function)
                .as_mut()
                .expect("Uninitialized closure in chunk.")
        };
        Vm::pop();
        Vm::push(closure as *mut dyn Obj);
        Vm::call(closure, 0)?;
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
        let b = Vm::peek(0);
        let b = b.as_obj().unwrap().as_rstring();
        let a = Vm::peek(1);
        let a = a.as_obj().unwrap().as_rstring();
        let c = ObjString::concat(a, b);
        let c = crate::objects::allocate_string(c.as_rstring()) as *mut dyn Obj;
        Vm::pop();
        Vm::pop();
        Vm::push(c);
    }
    fn run() -> Result<()> {
        let mut frame: *mut CallFrame = unsafe { &mut VM.frames[VM.frame_count - 1] };
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
                OpCode::SuperInvoke => {
                    let mut name = Vm::read_constant(frame);
                    let name = name
                        .as_obj_mut()
                        .expect("Expected object")
                        .as_string_mut()
                        .expect("Expected string");
                    let arg_count = Vm::read_byte(frame);
                    let mut superclass = Vm::pop();
                    let superclass = superclass
                        .as_obj_mut()
                        .expect("Expected object")
                        .as_class_mut()
                        .expect("Expected class");
                    Vm::invoke_from_class(superclass, name, arg_count)?;
                    frame = unsafe { &mut VM.frames[VM.frame_count - 1] };
                }
                OpCode::GetSuper => {
                    let mut name = Vm::read_constant(frame);
                    let name = name
                        .as_obj_mut()
                        .expect("Expected object")
                        .as_string_mut()
                        .expect("Expected string");
                    let mut superclass = Vm::pop();
                    let superclass = superclass
                        .as_obj_mut()
                        .expect("Expected object")
                        .as_class_mut()
                        .expect("Expected class");
                    Vm::bind_method(superclass, name)?;
                }
                OpCode::Inherit => {
                    let mut superclass = Vm::peek(1);
                    let Some(superclass) = superclass.as_obj_mut().unwrap().as_class_mut() else {
                        return Vm::runtime_error("Superclass must be a class.");
                    };
                    let mut subclass = Vm::peek(0);
                    let subclass = subclass.as_obj_mut().unwrap().as_class_mut().unwrap();
                    for (k, v) in &superclass.methods {
                        subclass.methods.insert(*k, *v);
                    }
                    Vm::pop(); // Subclass.
                }
                OpCode::Invoke => {
                    let method: *mut ObjString = {
                        let mut val = Vm::read_constant(frame);
                        val.as_obj_mut()
                            .expect("Expected object value.")
                            .as_string_mut()
                            .expect("Expected string.")
                    };
                    let arg_count = Vm::read_byte(frame);
                    Vm::invoke(method, arg_count)?;
                    frame = unsafe { &mut VM.frames[VM.frame_count - 1] };
                }
                OpCode::Method => {
                    let mut name = Vm::read_constant(frame);
                    Vm::define_method(
                        name.as_obj_mut()
                            .expect("Expect object value.")
                            .as_string_mut()
                            .expect("Expected string as class name"),
                    );
                }
                OpCode::GetProperty => {
                    let instance = Vm::peek(0);
                    let instance = instance.as_obj().expect("Expected Object value.");
                    let Some(instance) = instance.as_instance() else {
                       return Vm::runtime_error("Only instances have properties.");
                    };
                    let mut name = Vm::read_constant(frame);
                    let name = name
                        .as_obj_mut()
                        .expect("Expected object value.")
                        .as_string_mut()
                        .unwrap();
                    if let Some(value) = instance.fields.get(&(name as *mut ObjString)) {
                        Vm::pop(); // Instance.
                        Vm::push(*value);
                    } else if let Err(err) = Vm::bind_method(instance.klass, name) {
                        return Err(err);
                    }
                }
                OpCode::SetProperty => {
                    let mut instance = Vm::peek(1);
                    let instance = instance.as_obj_mut().expect("Expected Obj Value");
                    let Some(instance) = instance.as_instance_mut() else {
                        return Vm::runtime_error("Only instances have fields.");
                    };
                    let mut name = Vm::read_constant(frame);
                    let name = name
                        .as_obj_mut()
                        .expect("Expect object value.")
                        .as_string_mut()
                        .expect("Expected string");
                    instance.fields.insert(name as *mut ObjString, Vm::peek(0));
                    let value = Vm::pop();
                    Vm::pop();
                    Vm::push(value);
                }
                OpCode::Class => {
                    let mut name = Vm::read_constant(frame);
                    let name = name.as_obj_mut().unwrap().as_string_mut().unwrap();
                    Vm::push(ObjClass::new(name) as *mut dyn Obj);
                }
                OpCode::CloseUpvalue => {
                    Vm::close_upvalue(unsafe { VM.stack_top.sub(1) });
                    Vm::pop();
                }
                OpCode::GetUpvalue => {
                    let slot = Vm::read_byte(frame) as usize;
                    let value = unsafe {
                        *frame.as_ref().unwrap().closure().upvalues[slot]
                            .as_ref()
                            .unwrap()
                            .location
                    };
                    Vm::push(value);
                }

                OpCode::SetUpvalue => {
                    let slot = Vm::read_byte(frame) as usize;
                    unsafe {
                        std::ptr::write(
                            frame.as_ref().unwrap().closure().upvalues[slot]
                                .as_mut()
                                .unwrap()
                                .location,
                            Vm::peek(0),
                        );
                    }
                }

                OpCode::Closure => {
                    let mut function = Vm::read_constant(frame);
                    let function = function
                        .as_obj_mut()
                        .expect("Expected object value.")
                        .as_function_mut()
                        .expect("Expected funtion object.");
                    let closure = ObjClosure::new(function);
                    Vm::push(closure as *mut dyn Obj);
                    let closure = unsafe { closure.as_mut().unwrap() };
                    for i in 0..closure.upvalue_count {
                        let is_local = Vm::read_byte(frame);
                        let index = Vm::read_byte(frame) as usize;
                        if is_local == 1 {
                            closure.upvalues[i] = Vm::capture_upvalue(
                                &mut unsafe { frame.as_mut().unwrap() }.slots()[index],
                            )
                        } else {
                            closure.upvalues[i] =
                                unsafe { frame.as_mut().unwrap().closure().upvalues[index] }
                        }
                    }
                }
                OpCode::Call => {
                    let arg_count = Vm::read_byte(frame);
                    let p_val = Vm::peek(arg_count as isize);
                    Vm::call_value(p_val, arg_count as u32)?;
                    frame = unsafe { &mut VM.frames[VM.frame_count - 1] };
                    continue;
                }
                OpCode::Return => {
                    let result = Vm::pop();
                    unsafe {
                        Vm::close_upvalue(frame.as_ref().unwrap().slots);
                        VM.frame_count -= 1;
                        if VM.frame_count == 0 {
                            Vm::pop();
                            return Ok(());
                        }
                        let this_frame = frame.as_ref().unwrap();
                        VM.stack_top = this_frame.slots;
                        Vm::push(result);
                        frame = &mut VM.frames[VM.frame_count - 1];
                    }
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

                    if is_obj.0 && is_obj.1 {
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
                    let mut name = Vm::read_constant(frame);
                    let name = name.as_obj_mut().unwrap().as_string_mut().unwrap();
                    let table = unsafe { VM.globals.as_mut().unwrap() };
                    table.insert(name, Vm::peek(0));
                    Vm::pop();
                }
                OpCode::GetGlobal => {
                    let mut name = Vm::read_constant(frame);
                    let name = name.as_obj_mut().unwrap().as_string_mut().unwrap();
                    if let Some(value) =
                        unsafe { VM.globals.as_mut().unwrap().get(&(name as *mut ObjString)) }
                    {
                        Vm::push(*value);
                    } else {
                        return Vm::runtime_error(&format!("Undefine variable '{}'.", name));
                    }
                }
                OpCode::SetGlobal => {
                    let mut name = Vm::read_constant(frame);
                    let name = name.as_obj_mut().unwrap().as_string_mut().unwrap();
                    let table = unsafe { VM.globals.as_mut().unwrap() };
                    if let None = table.insert(name, Vm::peek(0)) {
                        table.remove(&(name as *mut ObjString));
                        return Vm::runtime_error(&format!("Undefined variable '{}'", name));
                    }
                }
                OpCode::GetLocal => {
                    let slot = Vm::read_byte(frame) as usize;
                    let frame = unsafe { frame.as_mut().unwrap() };
                    let val = frame.slots()[slot];
                    Vm::push(val);
                }
                OpCode::SetLocal => {
                    let slot = Vm::read_byte(frame) as usize;
                    let val = Vm::peek(0);
                    unsafe {
                        let frame = frame.as_mut().unwrap();
                        frame.slots()[slot] = val;
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
