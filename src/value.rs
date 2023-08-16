use std::{fmt::Display, ops::Not};

use crate::heap::Object;

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) enum Value {
    #[default]
    Nil,
    Number(i64),
    Bool(bool),
    Object(Object),
}
impl From<Object> for Value {
    fn from(value: Object) -> Self {
	Self::Object(value)
    }
}
impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Number(value)
    }
}
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
impl Not for Value {
    type Output = Value;
    fn not(self) -> Self::Output {
        Value::Bool(match self {
            Self::Nil => true,
            Self::Bool(b) => !b,
	    _ => false,
        })
    }
}
impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Number(n) => write!(f, "{n}"),
            Self::Bool(b) => write!(f, "{b}"),
	    Self::Object(o) => write!(f, "{o}"),
        }
    }
}

impl Value {
    /// Returns `true` if the value is [`Bool`].
    ///
    /// [`Bool`]: Value::Bool
    #[must_use]
    pub(crate) fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(..))
    }

    /// Returns `true` if the value is [`Number`].
    ///
    /// [`Number`]: Value::Number
    #[must_use]
    pub(crate) fn is_number(&self) -> bool {
        matches!(self, Self::Number(..))
    }

    /// Returns `true` if the value is [`Nil`].
    ///
    /// [`Nil`]: Value::Nil
    #[must_use]
    pub(crate) fn is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }

    pub(crate) fn try_into_number(self) -> Result<i64, Self> {
        if let Self::Number(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    pub(crate) fn try_into_bool(self) -> Result<bool, Self> {
        if let Self::Bool(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    /// Returns `true` if the value is [`Object`].
    ///
    /// [`Object`]: Value::Object
    #[must_use]
    pub(crate) fn is_object(&self) -> bool {
        matches!(self, Self::Object(..))
    }
}
