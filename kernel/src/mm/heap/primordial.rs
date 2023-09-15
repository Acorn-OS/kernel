use core::alloc::{AllocError, Allocator, Layout};
use core::ptr::{null_mut, NonNull};

use crate::mm::pmm;
use crate::util::locked::ManualLock;

static mut PTR: *mut u8 = null_mut();
static mut SIZE: usize = 0;
static mut IS_INIT: bool = false;

pub(super) unsafe fn alloc(layout: Layout) -> *mut u8 {
    if !IS_INIT {
        let pages = 8;
        SIZE = pages * pmm::PAGE_SIZE;
        PTR = pmm::alloc_pages(pages).virt().ptr();
        IS_INIT = true;
    }
    while PTR as usize % layout.align() != 0 {
        PTR = PTR.add(1);
        SIZE = SIZE.saturating_sub(1);
    }
    if SIZE == 0 || SIZE < layout.size() {
        return null_mut();
    }
    let ptr = PTR;
    SIZE -= layout.size();
    PTR = PTR.add(layout.size());
    ptr
}

pub(super) unsafe fn free(_: *mut u8, _: Layout) {}

#[derive(Clone, Copy)]
pub struct PrimordialAlloc;
static LOCK: ManualLock = ManualLock::new();

unsafe impl Allocator for PrimordialAlloc {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        LOCK.do_locked(|| unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                error!("primordial allocator failed");
                Err(AllocError)
            } else {
                Ok(NonNull::new_unchecked(core::ptr::from_raw_parts_mut(
                    ptr as *mut (),
                    layout.size(),
                )))
            }
        })
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        LOCK.do_locked(|| free(ptr.as_ptr(), layout))
    }
}
