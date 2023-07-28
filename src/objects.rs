use std::{fmt, mem::ManuallyDrop};

pub(crate) trait IsObj {
    fn get_obj_id() -> ObjId;
}
#[derive(Debug, Clone, Copy,PartialEq, Eq)]
pub(crate) enum ObjId {
    String,
    Function,
}

pub(crate) struct OwnedObject {
    id: ObjId,
    ptr: *mut u8,
}


impl OwnedObject {
    fn new<T: IsObj>(obj: T) -> OwnedObject {
	let id = T::get_obj_id();
	let b = Box::new(obj);
	let ptr = Box::into_raw(b).cast();
	Self {
	    id,
	    ptr,
	}
    }
}
impl From<&OwnedObject> for ObjRef {
    fn from(value: &OwnedObject) -> Self {
	Self {
	    id: value.id,
	    ptr: value.ptr.cast_const(),
	    
	}
    }
}
#[derive(Debug)]
pub struct ObjCastErr;
impl fmt::Display for ObjCastErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	write!(f, "{self:?}")
    }
}
impl std::error::Error for ObjCastErr {}
pub(crate) struct ObjRef {
    id: ObjId,
    ptr: *const u8,
}

impl ObjRef {
    pub(crate) fn get_ref<T: IsObj>(&self) -> Result<ManuallyDrop<&T>, ObjCastErr> {
	if T::get_obj_id() != self.id {
	    Err(ObjCastErr)
	} else {
	    let ptr = self.ptr.cast::<T>();
	    unsafe{
		// This unwrap is a problem for future me ;)
		Ok(ManuallyDrop::new(ptr.as_ref().unwrap()))
	    }
	}
    }
}
