//! This module outlines the interface to a heap object.
use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use super::{HeapObject, ObjMetaData, ObjString, ObjType};

pub(crate) trait IsObj {
    fn obj_id() -> ObjType;
}
/// A pointer to an object of unknown type.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) struct OpaquePtr(*const u8);
impl OpaquePtr {
    pub(crate) fn new<T: IsObj>(ptr: *const T) -> Self {
        Self(ptr.cast())
    }
}
/// A pointer to an object of known type.
#[derive(Debug)]
pub(crate) struct ObjPtr<T: IsObj>(*const ObjMetaData, *const T);
impl<T: IsObj> Default for ObjPtr<T> {
    fn default() -> Self {
        Self(std::ptr::null(), std::ptr::null())
    }
}
impl<T: IsObj> Hash for ObjPtr<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}
impl<T: IsObj> PartialEq for ObjPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}
impl<T: IsObj> Eq for ObjPtr<T> {}
impl<T: IsObj> Clone for ObjPtr<T> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}
impl<T: IsObj> Copy for ObjPtr<T> {}
impl<T: IsObj + Display> Display for ObjPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
impl<T: IsObj> ObjPtr<T> {
    pub(crate) fn new(obj: *const T, id: &ObjMetaData) -> Self {
        Self(id, obj)
    }
    pub(crate) fn from_opaque(ptr: OpaquePtr, id: &ObjMetaData) -> Self {
        Self(id, ptr.0.cast())
    }
    pub(crate) fn meta_data(&self) -> &ObjMetaData {
        unsafe { self.0.as_ref().unwrap() }
    }
    pub(crate) fn as_ref(&self) -> &T {
        unsafe { self.1.as_ref().unwrap() }
    }
    pub(crate) fn to_inner(self) -> *const T {
        self.1
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) struct Object(*const ObjMetaData, OpaquePtr);
impl Object {
    pub(crate) fn new(obj: &HeapObject) -> Self {
        Self(&obj.meta_data, obj.ptr)
    }
    pub(crate) fn obj_id(&self) -> ObjType {
        unsafe { self.0.as_ref().map(|t| t.id).unwrap() }
    }
    pub(crate) fn is_obj<T: IsObj>(&self) -> bool {
        unsafe { T::obj_id() == self.0.as_ref().map(|t| t.id).unwrap() }
    }
    pub(crate) fn as_obj<T: IsObj>(&self) -> ObjPtr<T> {
        let (r, id) = unsafe {
            let meta_data = self.0.as_ref().unwrap();
            let ptr = self.1;
            (ptr, meta_data)
        };
        ObjPtr::from_opaque(r, id)
    }
    pub(crate) fn obj_meta_data(&self) -> &ObjMetaData {
        unsafe { self.0.as_ref().unwrap() }
    }
    pub(crate) fn from_ptr<T: IsObj>(ptr: &ObjPtr<T>) -> Self {
        Self(ptr.meta_data(), OpaquePtr::new(ptr.to_inner()))
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.obj_meta_data().id {
            ObjType::String => {
                write!(f, "{}", self.as_obj::<ObjString>())
            }
        }
    }
}
