use crate::allocators::block::{self, BlockHeap, SpinBlockHeap};
use core::alloc::{GlobalAlloc, Layout};

struct Allocator {
    heap: SpinBlockHeap,
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        GlobalAlloc::alloc(&*self.heap.lock(), layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.heap.lock().dealloc(ptr, layout)
    }
}

#[global_allocator]
static KERNEL_ALLOCATOR: Allocator = Allocator {
    heap: unsafe { SpinBlockHeap::new(BlockHeap::from_parts(0 as *mut _, 0, 0)) },
};

pub fn alloc_bytes(bytes: usize) -> *mut u8 {
    if let Some((ptr, _)) = unsafe { KERNEL_ALLOCATOR.heap.lock().alloc(bytes, 1) } {
        ptr
    } else {
        core::ptr::null_mut()
    }
}

pub fn init() {
    util::once! {
        let block_ln2 = 4;
        let blocks = 4096 * 4096 * 16;
        *KERNEL_ALLOCATOR.heap.lock() = block::new(block_ln2, blocks, |amnt| unsafe {
            crate::mm::wm::reserve_amount(amnt)
        })
        .expect("failed to initialize kernel allocator");
    };
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!("allocation error: layout {layout:?}")
}
