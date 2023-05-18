struct GlobalAlloc;

unsafe impl core::alloc::GlobalAlloc for GlobalAlloc {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        todo!()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        todo!()
    }
}

#[global_allocator]
static GLOBAL_ALLOC: GlobalAlloc = GlobalAlloc;
