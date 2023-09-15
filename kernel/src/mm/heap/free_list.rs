use crate::mm::pmm;
use crate::mm::vmm::PAGE_SIZE;
use crate::util::locked::Locked;
use allocators::intrustive::free_list::{Error, IntrusiveFreeList};
use core::alloc::Layout;
use core::ptr::null_mut;

static FREE_LIST: Locked<IntrusiveFreeList<16>> = Locked::new(IntrusiveFreeList::new());

pub unsafe fn alloc(layout: Layout) -> *mut u8 {
    let mut locked = FREE_LIST.lock();
    match locked.alloc_layout(layout) {
        Ok(ok) => ok,
        Err(Error::InsufficientSpace) => {
            let aligned_size = align_ceil!(layout.size(), PAGE_SIZE);
            let pages = aligned_size / PAGE_SIZE;
            let vadr = pmm::alloc_pages(pages).virt();
            unsafe {
                FREE_LIST
                    .lock()
                    .push_region_unchecked(vadr.ptr(), pages * PAGE_SIZE);
            }
            match locked.alloc_layout(layout) {
                Ok(ok) => ok,
                Err(e) => {
                    error!("free list heap failed: {e}");
                    null_mut()
                }
            }
        }
        Err(_) => null_mut(),
    }
}

pub unsafe fn free(ptr: *mut u8, layout: Layout) {
    let mut locked = FREE_LIST.lock();
    locked.free_layout(ptr, layout);
}
