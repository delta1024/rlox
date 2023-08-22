//! This module provides concrete implementations of objects.
extern crate obj_derive;
use obj_derive::mark_obj;
use super::{IsObj, ObjPtr, OpaquePtr};
use std::{fmt::Display, ops::Deref};
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ObjType {
    String,
}
#[derive(Copy, Clone)]
pub(crate) struct ObjMetaData {
    pub(crate) id: ObjType,
}
#[derive(Clone)]
pub(crate) struct HeapObject {
    pub(super) meta_data: ObjMetaData,
    pub(super) ptr: OpaquePtr,
}
impl Display for HeapObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.meta_data.id {
            ObjType::String => write!(
                f,
                "{}",
                ObjPtr::<ObjString>::from_opaque(self.ptr, &self.meta_data)
            ),
        }
    }
}
impl Drop for HeapObject {
    fn drop(&mut self) {
        match self.meta_data.id {
            ObjType::String => {
                let obj_ptr = ObjPtr::<ObjString>::from_opaque(self.ptr, &self.meta_data)
                    .to_inner()
                    .cast_mut();
                unsafe {
                    _ = Box::from_raw(obj_ptr);
                }
            }
        }
    }
}
impl HeapObject {
    pub(crate) fn new<T: IsObj>(obj: T) -> Self {
        Self {
            meta_data: ObjMetaData { id: T::obj_id() },
            ptr: OpaquePtr::new(Box::into_raw(Box::new(obj))),
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Debug, Default)]
#[mark_obj(String)]
pub(crate) struct ObjString(String);
impl Deref for ObjString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0[..]
    }
}
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


