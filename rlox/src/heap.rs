use std::collections::{HashMap, LinkedList};

use crate::{
    heap_objects::ObjString,
    objects::{IsObj, ObjRef, OwnedObject},
};

pub(crate) struct Heap {
    strings: HashMap<String, ObjRef>,
    objects: LinkedList<OwnedObject>,
}

impl Heap {
    pub(crate) fn new() -> Self {
        Self {
            strings: HashMap::new(),
            objects: LinkedList::new(),
        }
    }
    pub(crate) fn allocate_object<T: IsObj>(&mut self, obj: T) -> ObjRef {
        let obj = OwnedObject::new(obj);
        let obj_ref: ObjRef = (&obj).into();
        self.objects.push_back(obj);
        obj_ref
    }
    pub(crate) fn allocate_string(&mut self, string: impl ToString) -> ObjRef {
        let string = string.to_string();
        if let Some(string) = self.strings.get(&string) {
            return *string;
        }
        let obj = self.allocate_object(ObjString(string.clone()));
        _ = self.strings.insert(string.clone(), obj);
        *self.strings.get(&string).unwrap()
    }
}
