use core::alloc::{GlobalAlloc, Layout};

use spin::Mutex;

use super::heap::Heap;
use crate::arch;

enum Allocator {
    Alloc(spin::Mutex<Heap<12>>),
    Uninit,
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match self {
            Allocator::Alloc(alloc) => GlobalAlloc::alloc(&*alloc.lock(), layout),
            Allocator::Uninit => panic!("attempting to allocate with uninitialized allocator."),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        match self {
            Allocator::Alloc(alloc) => GlobalAlloc::dealloc(&*alloc.lock(), ptr, layout),
            Allocator::Uninit => panic!("attempting to deallocate with uninitialized allocator."),
        }
    }
}

#[global_allocator]
static mut KERNEL_ALLOCATOR: Allocator = Allocator::Uninit;

pub unsafe fn init() {
    KERNEL_ALLOCATOR = Allocator::Alloc(Mutex::new(Heap::new()));
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!("allocation error: layout {layout:?}")
}
