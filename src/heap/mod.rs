pub(crate) mod objects;
pub(crate) mod heap_objects;
pub(crate) mod allocator;

use std::collections::LinkedList;

pub(crate) use objects::*;
pub(crate) use heap_objects::*;
pub(crate) use allocator::*;

pub(crate) struct Heap {
    objects: LinkedList<HeapObject>,
}

impl Heap {
    pub(crate) fn new() -> Self {
	Self{
	    objects: LinkedList::new(),
	}
    }
    pub(crate) fn allocate_obj<T: IsObj>(&mut self, obj: T) -> Object {
	let heap_obj =  HeapObject::new(obj);
	let n = {self.objects.push_back(heap_obj); self.objects.back().unwrap()};
	Object::new(n)
    }
    pub(crate) fn allocator(&mut self) -> Allocator {
	Allocator::new(self)
    }
}
