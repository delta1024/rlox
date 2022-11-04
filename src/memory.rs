use crate::{
    compiler::mark_compiler_roots,
    objects::{Obj, ObjClosure, ObjString, ObjType},
    value::Value,
    vm::VM,
};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    collections::HashMap,
};

static mut RUNNING: bool = false;
pub struct GarbageCollector;
const GC_HEAP_GROW_FACTOR: usize = 2;

impl GarbageCollector {
    unsafe fn collect_garbage(&self) {
        if RUNNING {
            return;
        }
        RUNNING = true;
        #[cfg(feature = "log_gc")]
        let befor = {
            println!("-- gc begin");
            VM.bytes_allocated
        };

        self.mark_roots();
        self.trace_references();
        self.table_remove_white();
        self.sweep();

        VM.next_gc = VM.bytes_allocated * GC_HEAP_GROW_FACTOR;
        #[cfg(feature = "log_gc")]
        {
            println!("-- gc end");
            println!(
                "   collected {} bytes (from {} to {}) next at {}",
                VM.bytes_allocated - befor,
                befor,
                VM.bytes_allocated,
                VM.next_gc
            );
        }
        RUNNING = false;
    }

    unsafe fn sweep(&self) {
        let list = VM
            .objects
            .take()
            .expect("uninitialized vm")
            .into_iter()
            .filter_map(|mut val| {
                if val.is_marked() {
                    val.set_mark(false);
                    Some(val)
                } else {
                    None
                }
            })
            .collect();
        _ = VM.objects.insert(list);
    }

    unsafe fn table_remove_white(&self) {
        let table = VM.strings.take().unwrap_or_default();
        _ = VM.strings.insert(
            table
                .into_iter()
                .filter(|(_, x)| x.as_ref().is_marked())
                .collect(),
        );
    }
    unsafe fn mark_roots(&self) {
        let slots = {
            let offset = VM.stack_top.offset_from(&VM.stack[0]) as usize;
            std::slice::from_raw_parts_mut(&mut VM.stack[0], offset)
        };
        for slot in &mut slots[..] {
            Self::mark_value(slot);
        }

        for i in &mut VM.frames[..VM.frame_count] {
            Self::mark_obj(i.closure as *mut dyn Obj);
        }

        let mut upvalue = VM.open_upvalues;
        while !upvalue.is_null() {
            let obj = upvalue.as_mut().unwrap();
            Self::mark_obj(obj);
            upvalue = obj.next;
        }
        Self::mark_table(VM.globals.as_mut().expect("Unintialized vm."));
        mark_compiler_roots();
    }
    unsafe fn trace_references(&self) {
        let gray_stack = VM.gray_stack.as_mut().expect("vm not initialized");
        while let Some(obj) = gray_stack.pop() {
            Self::blacken_object(obj);
        }
    }
    unsafe fn blacken_object(object: &mut dyn Obj) {
        #[cfg(feature = "log_gc")]
        {
            println!("{:?} blacken {}", object as *mut dyn Obj, object);
        }
        match object.id() {
            ObjType::Upvalue => {
                let object = object.as_upvalue_mut().unwrap();
                Self::mark_value(&mut object.closed);
            }
            ObjType::Function => {
                let funciton = object.as_function_mut().unwrap();
                Self::mark_obj(funciton.name);
                Self::mark_array(&mut funciton.chunk.constants)
            }
            ObjType::Closure => {
                let closure = object.as_closure_mut().unwrap();
                Self::mark_obj(closure);
                for i in &closure.upvalues {
                    Self::mark_obj(*i);
                }
            }
            ObjType::None | ObjType::Native | ObjType::String => return,
        }
    }
    unsafe fn mark_array(array: &mut Vec<Value>) {
        for i in array {
            Self::mark_value(i);
        }
    }
    unsafe fn mark_table(table: &mut HashMap<*mut ObjString, Value>) {
        for (k, v) in table {
            Self::mark_obj(*k);
            Self::mark_value(v);
        }
    }
    unsafe fn mark_value(value: *mut Value) {
        if !value.is_null() && value.as_mut().unwrap().as_obj_mut().is_ok() {
            let obj: *mut dyn Obj = value.as_mut().unwrap().as_obj_mut().unwrap();
            Self::mark_obj(obj as *mut dyn Obj);
        }
    }

    pub unsafe fn mark_obj(obj: *mut dyn Obj) {
        if obj.is_null() {
            return;
        }
        let obj = obj.as_mut().unwrap();
        if obj.is_marked() {
            return;
        }
        #[cfg(feature = "log_gc")]
        {
            let obj_ptr = obj as *mut dyn Obj;
            println!("{:?} mark {}", obj_ptr, obj);
        }
        obj.set_mark(true);
        if VM.gray_stack.is_none() {
            _ = VM.gray_stack.insert(Vec::new());
        }
        VM.gray_stack
            .as_mut()
            .expect("vm not initialized")
            .push(obj);
    }
}

unsafe impl GlobalAlloc for GarbageCollector {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        VM.bytes_allocated += new_size - layout.size();
        if new_size > layout.size() {
            #[cfg(feature = "stress_gc")]
            self.collect_garbage();

            if VM.bytes_allocated > VM.next_gc {
                self.collect_garbage();
            }
        }
        System.realloc(ptr, layout, new_size)
    }
}
