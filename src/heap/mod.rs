pub(crate) mod allocator;
pub(crate) mod heap_objects;
pub(crate) mod objects;

use std::collections::{HashMap, LinkedList};

pub(crate) use allocator::*;
pub(crate) use heap_objects::*;
pub(crate) use objects::*;

use crate::value::Value;

pub(crate) struct Heap {
    strings: HashMap<String, ObjPtr<ObjString>>,
    globals: HashMap<ObjPtr<ObjString>, Value>,
    objects: LinkedList<HeapObject>,
}

impl Heap {
    pub(crate) fn new() -> Self {
        Self {
            objects: LinkedList::new(),
	    globals: HashMap::new(),
            strings: HashMap::new(),
        }
    }
    pub(crate) fn allocate_obj<T: IsObj>(&mut self, obj: T) -> Object {
        let heap_obj = HeapObject::new(obj);
        let n = {
            self.objects.push_back(heap_obj);
            self.objects.back().unwrap()
        };
        Object::new(n)
    }
    pub(crate) fn alloacte_string<T: ToString>(&mut self, string: T) -> Object {
        let key = string.to_string();
        if let Some(obj) = self.strings.get(&key) {
            return Object::from_ptr(obj);
        }
        let obj = self.allocate_obj(ObjString::new(key.clone()));
        _ = self.strings.insert(key, obj.as_obj());
        obj
    }
    pub(crate) fn allocator(&mut self) -> Allocator {
        Allocator::new(self)
    }
}
