use std::{
    fmt::{Debug, Display},
    marker::PhantomPinned,
    pin::Pin,
};
pub trait Obj: Debug + Display {
    fn id(&self) -> ObjType {
        ObjType::None
    }

    fn as_string(&self) -> Option<&ObjString> {
        None
    }

    fn as_rstring(&self) -> &str {
        ""
    }
}

#[derive(PartialEq)]
pub enum ObjType {
    String,
    None,
}
#[derive(Debug, Eq, Hash, PartialOrd, Ord)]
pub struct ObjString {
    chars: Vec<u8>,
    _marker: PhantomPinned,
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
            _marker: PhantomPinned,
        })
    }

    pub fn concat(a: &str, b: &str) -> Pin<Box<Self>> {
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
        Box::pin(ObjString {
            chars: n,
            _marker: PhantomPinned,
        })
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
