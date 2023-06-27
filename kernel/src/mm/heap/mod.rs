pub mod primordial;

mod error;
mod free_list;

use alloc::alloc::Global;
use allocators::bitmap::BitMapPtrAllocator;
use core::alloc::{Allocator, Layout};
use core::ptr::NonNull;

pub use error::{Error, Result};

type NodeAllocator = BitMapPtrAllocator<3>;
type AllocatorTy = allocators::freelist::FreeListAllocator<NodeAllocator>;

struct GlobalAlloc {
    alloc_fn: unsafe fn(Layout) -> *mut u8,
    free_fn: unsafe fn(*mut u8, Layout),
}

unsafe impl core::alloc::GlobalAlloc for GlobalAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        (self.alloc_fn)(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        (self.free_fn)(ptr, layout)
    }
}

#[global_allocator]
static mut GLOBAL_ALLOC: GlobalAlloc = GlobalAlloc {
    alloc_fn: primordial::alloc,
    free_fn: primordial::free,
};

#[inline]
pub unsafe fn alloc_bytes(count: usize) -> NonNull<u8> {
    debug_assert!(count <= isize::MAX as usize);
    Global
        .allocate(Layout::from_size_align_unchecked(count, 1))
        .expect("failed")
        .cast::<u8>()
}

#[inline]
pub unsafe fn free_bytes(ptr: NonNull<u8>, count: usize) {
    debug_assert!(count <= isize::MAX as usize);
    Global.deallocate(ptr, Layout::from_size_align_unchecked(count, 1))
}

#[inline]
pub fn alloc_layout(layout: Layout) -> NonNull<u8> {
    Global
        .allocate(layout)
        .expect(&format!("failed to allocate with layout '{layout:?}'"))
        .cast::<u8>()
}

#[inline]
pub fn free_layout(ptr: NonNull<u8>, layout: Layout) {
    unsafe { Global.deallocate(ptr, layout) }
}

#[inline]
pub fn alloc<T>(val: T) -> NonNull<T> {
    let ptr = alloc_layout(Layout::new::<T>()).cast::<T>();
    unsafe {
        ptr.as_ptr().write(val);
    }
    ptr
}

#[inline]
pub fn free<T>(ptr: *const T) {
    debug_assert!(!ptr.is_null());
    let layout = Layout::new::<T>();
    free_layout(unsafe { NonNull::new_unchecked(ptr as *mut u8) }, layout)
}

/// called when the kernel process has been properly initialized
pub unsafe fn init() {
    GLOBAL_ALLOC.alloc_fn = free_list::alloc;
    GLOBAL_ALLOC.free_fn = free_list::free;
}
