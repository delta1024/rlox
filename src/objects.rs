use std::{
    fmt::{Debug, Display},
    pin::Pin,
};

use crate::{chunk::Chunk, value::Value, vm::Vm};

pub trait Obj: Debug + Display + Unpin {
    fn id(&self) -> ObjType;
    fn as_rstring(&self) -> &str;
    fn as_string(&self) -> Option<&ObjString> {
        None
    }

    fn as_native(&self) -> Option<&ObjNative> {
        None
    }

    fn as_function_mut(&mut self) -> Option<&mut ObjFunction> {
        None
    }

    fn as_function(&self) -> Option<&ObjFunction> {
        None
    }

    fn as_closure(&self) -> Option<&ObjClosure> {
        None
    }
    fn as_closure_mut(&mut self) -> Option<&mut ObjClosure> {
        None
    }
}

#[derive(PartialEq)]
pub enum ObjType {
    Function,
    Native,
    String,
    Closure,
    None,
}
#[derive(Debug)]
pub struct ObjClosure {
    pub function: *mut ObjFunction,
}

impl ObjClosure {
    pub fn new(function: *mut ObjFunction) -> *mut ObjClosure {
        Vm::allocate_obj(Self { function }) as *mut ObjClosure
    }
}
impl Display for ObjClosure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { write!(f, "{}", self.function.as_ref().unwrap()) }
    }
}
impl Obj for ObjClosure {
    fn id(&self) -> ObjType {
        ObjType::Closure
    }
    fn as_rstring(&self) -> &str {
        todo!()
    }
    fn as_closure(&self) -> Option<&ObjClosure> {
        Some(self)
    }
    fn as_closure_mut(&mut self) -> Option<&mut ObjClosure> {
        Some(self)
    }
}
#[derive(Debug)]
pub struct ObjFunction {
    pub arity: u32,
    pub chunk: Chunk,
    pub name: *const ObjString,
}
impl ObjFunction {
    pub fn new(name: *const ObjString) -> *mut ObjFunction {
        let function = ObjFunction {
            arity: 0,
            chunk: Chunk::new(),
            name,
        };
        Vm::allocate_obj(function) as *mut ObjFunction
    }
}
impl Display for ObjFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.as_rstring() /*.as_rstring()*/)
    }
}
impl Obj for ObjFunction {
    fn id(&self) -> ObjType {
        ObjType::Function
    }

    fn as_function_mut(&mut self) -> Option<&mut ObjFunction> {
        Some(self)
    }

    fn as_function(&self) -> Option<&ObjFunction> {
        Some(self)
    }
    fn as_rstring(&self) -> &str {
        match unsafe { self.name.as_ref() } {
            Some(s) => s.as_rstring(),
            None => "script",
        }
    }
}
pub type NativeFn = fn(args: &[Value]) -> Value;
pub struct ObjNative {
    pub function: NativeFn,
}

impl ObjNative {
    pub fn new(function: NativeFn) -> *mut dyn Obj {
        Vm::allocate_obj(ObjNative { function })
    }
}

impl Debug for ObjNative {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

impl Display for ObjNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}

impl Obj for ObjNative {
    fn id(&self) -> ObjType {
        ObjType::Native
    }
    fn as_rstring(&self) -> &str {
        unimplemented!()
    }
    fn as_native(&self) -> Option<&ObjNative> {
        Some(self)
    }
}

#[derive(Debug, Eq, Hash, PartialOrd, Ord)]
pub struct ObjString {
    chars: Vec<u8>,
}

impl PartialEq for ObjString {
    fn eq(&self, other: &Self) -> bool {
        let n = self.as_rstring();
        let other = other.as_rstring();
        n == other
    }
}
impl ObjString {
    pub fn new(string: &str) -> Pin<Box<Self>> {
        Box::pin(ObjString {
            chars: string.chars().fold(Vec::new(), |mut v, c| {
                v.push(c as u8);
                v
            }),
        })
    }

    pub fn concat(a: &str, b: &str) -> Self {
        let mut n = Vec::new();
        for i in a.chars() {
            n.push(i as u8);
        }
        // Remove the closing quote.
        n.pop();
        let mut b = b.chars();
        // remove the opening quote
        b.next();
        for i in b {
            n.push(i as u8);
        }
        ObjString { chars: n }
    }
}

impl Display for ObjString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_rstring())
    }
}

impl Obj for ObjString {
    fn id(&self) -> ObjType {
        ObjType::String
    }

    fn as_string(&self) -> Option<&ObjString> {
        Some(self)
    }

    fn as_rstring(&self) -> &str {
        let slice = &self.chars[..self.chars.len()];
        std::str::from_utf8(slice).unwrap()
    }
}
pub fn allocate_string(key: &str) -> *mut ObjString {
    let table = unsafe {
        let table = crate::vm::VM.strings.as_mut();
        table.unwrap()
    };

    if let Some(string) = table.get_mut(key) {
        let n = Pin::get_mut(string.as_mut());
        n
    } else {
        table.insert(key.to_string(), ObjString::new(key));
        let n = table.get_mut(key).unwrap();
        let n = Pin::get_mut(n.as_mut());
        n
    }
}
