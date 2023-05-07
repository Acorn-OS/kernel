use super::pmm;
use alloc::alloc::Global;
use bitmap::BitMapPtrAllocator;
use core::alloc::{Allocator, Layout};
use core::ptr::{null_mut, NonNull};

const PAGE_COUNT: usize = 256;

type NodeAllocator = BitMapPtrAllocator<3>;
type AllocatorTy = freelist::FreeListAllocator<NodeAllocator>;

#[global_allocator]
static mut GLOBAL_ALLOC: AllocatorTy =
    unsafe { AllocatorTy::with_allocator(NodeAllocator::new(null_mut(), 0, null_mut())) };

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

pub fn dealloc_layout(ptr: NonNull<u8>, layout: Layout) {
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

pub fn dealloc<T>(ptr: *const T) {
    debug_assert!(!ptr.is_null());
    let layout = Layout::new::<T>();
    dealloc_layout(unsafe { NonNull::new_unchecked(ptr as *mut u8) }, layout)
}

pub unsafe fn init() {
    let bitmap_len = pmm::PAGE_SIZE;
    let bitmap_base = pmm::alloc_pages(bitmap_len.div_ceil(pmm::PAGE_SIZE * 8));
    let bitmap_alloc =
        pmm::alloc_pages((bitmap_len * NodeAllocator::PAGE_SIZE).div_floor(pmm::PAGE_SIZE));
    GLOBAL_ALLOC = AllocatorTy::with_allocator(BitMapPtrAllocator::new(
        bitmap_base.as_virt_ptr(),
        bitmap_len,
        bitmap_alloc.as_virt_ptr(),
    ));
    let virtual_base = pmm::alloc_pages(PAGE_COUNT).virt_adr();
    info!("heap virtual base 0x{virtual_base:016x} and 0x{PAGE_COUNT:x} pages");
    GLOBAL_ALLOC
        .push_region(virtual_base, PAGE_COUNT * pmm::PAGE_SIZE)
        .expect("unable to push region into global heap");
}
