use std::fmt::Display;
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
pub(crate) enum Value {
    #[default]
    Nil,
    Int(i64),
    Bool(bool),
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
	Self::Int(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
	Self::Bool(value)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Value::*;
        match self {
            Nil => write!(f, "nil"),
            Int(v) => write!(f, "{v}"),
            Bool(v) => write!(f, "{v}"),
        }
    }
}
