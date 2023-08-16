use std::fmt::Display;

use super::{IsObj, ObjPtr, ObjString, OpaquePtr};
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ObjType {
    String,
}
#[derive(Copy, Clone)]
pub(crate) struct ObjMetaData {
    id: ObjType,
}
#[derive(Clone)]
pub(crate) struct HeapObject {
    meta_data: ObjMetaData,
    ptr: OpaquePtr,
}
impl Display for HeapObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.meta_data.id {
            ObjType::String => write!(f, "{}", ObjPtr::<ObjString>::from_opaque(self.ptr)),
        }
    }
}
impl Drop for HeapObject {
    fn drop(&mut self) {
        match self.meta_data.id {
            ObjType::String => {
                let obj_ptr = ObjPtr::<ObjString>::from_opaque(self.ptr)
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
        let ptr = ObjPtr::new(obj);
        Self {
            meta_data: ObjMetaData { id: T::obj_id() },
            ptr: ptr.into(),
        }
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) struct Object(*const HeapObject);
impl Object {
    pub(crate) fn new(obj: &HeapObject) -> Self {
        Self(obj)
    }
    pub(crate) fn obj_id(&self) -> ObjType {
        unsafe { self.0.as_ref().map(|t| t.meta_data.id).unwrap() }
    }
    pub(crate) fn is_obj<T: IsObj>(&self) -> bool {
        unsafe { T::obj_id() == self.0.as_ref().map(|t| t.meta_data.id).unwrap() }
    }
    pub(crate) fn as_obj<T: IsObj>(&self) -> ObjPtr<T> {
        let r = unsafe { self.0.as_ref().map(|h| h.ptr).unwrap() };
        ObjPtr::from_opaque(r)
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", unsafe { self.0.as_ref().unwrap() })
    }
}
