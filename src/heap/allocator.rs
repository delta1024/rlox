use super::{Heap, IsObj, Object};
#[derive(Debug)]
pub(crate) struct Allocator {
    heap_ptr: *mut Heap,
}
impl Allocator {
    pub(crate) fn new(heap_ptr: *mut Heap) -> Self {
	Self{
	    heap_ptr
	}
    }
    pub(crate) fn allocate_obj<T: IsObj>(&self, obj: T) -> Object {
	unsafe {
	    self.heap_ptr.as_mut().map(|heap| heap.allocate_obj(obj)).unwrap()
	}
    }
}
