use std::{
    fmt,
    ops::{Add, Div, Mul, Neg, Sub},
};

pub use crate::error::ValueError as Error;
#[derive(Debug, Clone, Copy)]
pub enum Value {
    Number(f64),
    Null,
}

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

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut n = String::new();
        match self {
            Self::Number(m) => {
                n = format!("{}", m);
            }
            Self::Null => n.push_str("null"),
        }
        write!(f, "{}", n)
    }
}

impl Neg for Value {
    type Output = Value;
    fn neg(self) -> Self::Output {
        Value::Number(-(f64::try_from(self).expect("Value must be a number")))
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
