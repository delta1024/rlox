use std::fmt::Display;

use super::ObjType;

pub(crate) trait IsObj {
    fn obj_id() -> ObjType;
}
#[repr(transparent)]
#[derive(Copy, Clone)]
pub(crate) struct OpaquePtr(*const u8);


#[repr(transparent)]
#[derive(Copy, Clone)]
pub(crate) struct ObjPtr<T: IsObj>(*const T);
impl<T: IsObj + Display> Display for ObjPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	write!(f, "{}", self.as_ref())
    }
}
impl<T: IsObj> ObjPtr<T> {
    pub(crate) fn new(obj: T) -> Self {
	Self(Box::into_raw(Box::new(obj)))
    }
    pub(crate) fn from_opaque(ptr: OpaquePtr) -> Self {
	Self(ptr.0.cast())
    }
    pub(crate) fn as_ref(&self) -> &T {
	unsafe {
	    self.0.as_ref().unwrap()    
	}
    }
    pub(crate) fn to_inner(self) -> *const T{
	self.0
    }
}
impl<T: IsObj> From<ObjPtr<T>> for OpaquePtr {
    fn from(value: ObjPtr<T>) -> Self {
	Self(value.0.cast())
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Default)]
pub(crate) struct ObjString(String);
impl Display for ObjString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	write!(f, "{}", self.0)
    }
}
impl ObjString {
    pub(crate) fn new(s: impl ToString) -> Self {
	Self(s.to_string())
    }
}

impl IsObj for ObjString {
    fn obj_id() -> ObjType {
	ObjType::String
    }
}
