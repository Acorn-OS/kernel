mod error;

use crate::mm::pmm;
use alloc::alloc::Global;
use allocators::bitmap::BitMapPtrAllocator;
use allocators::freelist::{Error as FreeListError, FreeList};
use core::alloc::{Allocator, Layout};
use core::ptr::{null_mut, NonNull};

pub use error::{Error, Result};

type NodeAllocator = BitMapPtrAllocator<3>;
type AllocatorTy = allocators::freelist::FreeListAllocator<NodeAllocator>;

struct GlobalAlloc {
    allocator: AllocatorTy,
}

unsafe impl core::alloc::GlobalAlloc for GlobalAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.allocator.lock();
        match FreeList::alloc_layout(&mut allocator, layout) {
            Ok(ptr) => ptr,
            Err(FreeListError::InsufficientSpace) => {
                let pages = pages!(layout.size());
                allocator
                    .push_region(pmm::alloc_pages(pages).virt().adr(), pages * pmm::PAGE_SIZE)
                    .expect("failed to allocate additional nodes");
                allocator.alloc_layout(layout).unwrap_or(null_mut())
            }
            Err(_) => null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        debug_assert!(!ptr.is_null());
        self.allocator
            .deallocate(NonNull::new_unchecked(ptr), layout)
    }
}

#[global_allocator]
static mut GLOBAL_ALLOC: GlobalAlloc = GlobalAlloc {
    allocator: unsafe {
        AllocatorTy::with_allocator(NodeAllocator::new(null_mut(), 0, null_mut()))
    },
};

pub unsafe fn alloc_bytes(count: usize) -> NonNull<u8> {
    debug_assert!(count <= isize::MAX as usize);
    Global
        .allocate(Layout::from_size_align_unchecked(count, 1))
        .expect("failed")
        .cast::<u8>()
}

pub unsafe fn free_bytes(ptr: NonNull<u8>, count: usize) {
    debug_assert!(count <= isize::MAX as usize);
    Global.deallocate(ptr, Layout::from_size_align_unchecked(count, 1))
}

pub fn alloc_layout(layout: Layout) -> NonNull<u8> {
    Global
        .allocate(layout)
        .expect(&format!("failed to allocate with layout '{layout:?}'"))
        .cast::<u8>()
}

pub fn free_layout(ptr: NonNull<u8>, layout: Layout) {
    unsafe { Global.deallocate(ptr, layout) }
}

pub fn alloc<T>(val: T) -> NonNull<T> {
    let layout = Layout::new::<T>();
    let ptr = alloc_layout(layout).cast::<T>();
    unsafe {
        ptr.as_ptr().write(val);
    }
    ptr
}

pub fn free<T>(ptr: *const T) {
    debug_assert!(!ptr.is_null());
    let layout = Layout::new::<T>();
    free_layout(unsafe { NonNull::new_unchecked(ptr as *mut u8) }, layout)
}

pub unsafe fn init() {
    let bitmap_len = pmm::PAGE_SIZE;
    let bitmap_base = pmm::alloc_pages(bitmap_len.div_ceil(pmm::PAGE_SIZE * 8));
    let bitmap_alloc = pmm::alloc_pages(pages!(bitmap_len * NodeAllocator::PAGE_SIZE));
    GLOBAL_ALLOC = GlobalAlloc {
        allocator: AllocatorTy::with_allocator(BitMapPtrAllocator::new(
            bitmap_base.virt().ptr(),
            bitmap_len,
            bitmap_alloc.virt().ptr(),
        )),
    };
}
