use spin::Mutex;

struct GlobalAlloc {
    lock: Mutex<()>,
}

unsafe impl core::alloc::GlobalAlloc for GlobalAlloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let _lock = self.lock.lock();
        let ptr = syscall::malloc(layout.size() + layout.align());
        let mask = layout.align() as u64 - 1;
        let adr = (ptr as u64 + mask) & !mask;
        adr as *mut _
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let _lock = self.lock.lock();
        debug_assert!(ptr as u64 & (layout.align() as u64 - 1) == 0);
        syscall::free(ptr, layout.size())
    }
}

#[global_allocator]
static GLOBAL_ALLOC: GlobalAlloc = GlobalAlloc {
    lock: spin::Mutex::new(()),
};
