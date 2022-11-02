use std::alloc::{GlobalAlloc, Layout, System};

pub struct GarbageCollector;

unsafe impl GlobalAlloc for GarbageCollector {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}
