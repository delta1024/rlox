use std::{
    fmt,
    ops::{Add, Div, Mul, Neg, Sub},
};

pub use crate::error::ValueError as Error;
use crate::objects::{Obj, ObjType};
#[derive(Debug, Clone, Copy, PartialOrd)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Obj(*mut dyn Obj),
    Null,
}
impl Value {
    pub fn as_obj(&self) -> Result<&dyn Obj, Error> {
        if let Self::Obj(o) = self {
            unsafe { Ok((*o).as_ref().unwrap()) }
        } else {
            Err(Error)
        }
    }
    pub fn as_obj_mut(&mut self) -> Result<&mut dyn Obj, Error> {
        if let Self::Obj(o) = self {
            unsafe { Ok((*o).as_mut().unwrap()) }
        } else {
            Err(Error)
        }
    }
}
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs == rhs,
            (Self::Null, Self::Null) => true,
            (Self::Bool(lhs), Self::Bool(rhs)) => lhs == rhs,
            (Self::Obj(lhs), Self::Obj(rhs)) => {
                let lhs = unsafe { lhs.as_ref().unwrap() };

                let rhs = unsafe { rhs.as_ref().unwrap() };

                match (lhs.id(), rhs.id()) {
                    (ObjType::String, ObjType::String) => lhs.as_string() == rhs.as_string(),
                    (ObjType::None, ObjType::None) => true,
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Null => true,
            Value::Bool(b) => !b,
            _ => false,
        }
    }
}
impl Eq for Value {}

impl TryFrom<Value> for f64 {
    type Error = crate::value::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Number(n) = value {
            Ok(n)
        } else {
            Err(Error)
        }
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

impl TryFrom<Value> for bool {
    type Error = self::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Bool(n) => Ok(n),
            _ => Err(Error),
        }
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<*mut dyn Obj> for Value {
    fn from(s: *mut dyn Obj) -> Self {
        Self::Obj(s)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut n = String::new();
        match self {
            Self::Number(m) => {
                n = format!("{}", m);
            }
            Self::Bool(s) => {
                n = format!("{:?}", s);
            }
            Self::Obj(ptr) => {
                let obj = unsafe { ptr.as_ref().unwrap() };
                let temp = format!("{}", obj);
                n.push_str(&temp);
            }
            Self::Null => n.push_str("null"),
        }
        write!(f, "{}", n)
    }
}

impl Neg for Value {
    type Output = Value;
    fn neg(self) -> Self::Output {
        match self {
            Self::Number(n) => Self::Number(n),
            _ => unreachable!(),
        }
    }
}

impl Add for Value {
    type Output = Value;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Self::Number(lhs + rhs),
            _ => unreachable!(),
        }
    }
}

impl Sub for Value {
    type Output = Value;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Self::Number(lhs - rhs),
            _ => unreachable!(),
        }
    }
}

impl Mul for Value {
    type Output = Value;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Self::Number(lhs * rhs),
            _ => unreachable!(),
        }
    }
}

impl Div for Value {
    type Output = Value;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Self::Number(lhs / rhs),
            _ => unreachable!(),
        }
    }
}
