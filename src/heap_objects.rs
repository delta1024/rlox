use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use crate::{
    byte_code::{Chunk, ChunkBuilder},
    objects::{IsObj, ObjId, ObjRef},
};

pub(crate) struct ObjString(pub String);
impl Deref for ObjString {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ObjString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IsObj for ObjString {
    fn get_obj_id() -> crate::objects::ObjId {
        ObjId::String
    }
}

impl Display for ObjString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

pub(crate) struct ObjFunction {
    name: ObjRef,
    pub(crate) chunk: Chunk,
}

impl ObjFunction {
    pub(crate) fn new(name: ObjRef, chunk: Chunk) -> Self {
        Self {
            name,
            chunk
        }
    }
}
impl Display for ObjFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn<{}>", self.name)
    }
}
impl IsObj for ObjFunction {
    fn get_obj_id() -> ObjId {
        ObjId::Function
    }
}
