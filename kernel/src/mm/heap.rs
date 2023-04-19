use super::pmm::Page;
use super::{hhdm, pmm};
use core::alloc::{Allocator, GlobalAlloc, Layout};

pub const PAGE_SIZE: usize = hhdm::PAGE_SIZE;
pub const PAGE_EXP: usize = hhdm::PAGE_EXP;
pub const PAGE_COUNT: usize = 256;
static mut VIRTUAL_BASE: u64 = 0;

#[derive(Default)]
struct NodeAllocator;

unsafe impl Allocator for NodeAllocator {
    /* TODO: optimize for space */
    fn allocate(
        &self,
        layout: Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
        Ok(unsafe {
            core::ptr::NonNull::new_unchecked(core::slice::from_raw_parts_mut(
                hhdm::to_virt(pmm::alloc_pages(1)) as *mut u8,
                layout.size(),
            ))
        })
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, _layout: Layout) {
        pmm::free_pages(hhdm::to_phys(ptr.as_ptr() as *mut Page), 1)
    }
}

type AllocatorTy = freelist::FreeListAllocator<NodeAllocator>;

#[global_allocator]
static GLOBAL_ALLOC: AllocatorTy = unsafe { AllocatorTy::with_allocator(NodeAllocator) };

pub unsafe fn alloc_bytes(count: usize) -> *mut u8 {
    debug_assert!(count <= isize::MAX as usize);
    GlobalAlloc::alloc(&GLOBAL_ALLOC, Layout::from_size_align_unchecked(count, 1))
}

pub unsafe fn free_bytes(ptr: *mut u8, count: usize) {
    debug_assert!(count <= isize::MAX as usize);
    GlobalAlloc::dealloc(
        &GLOBAL_ALLOC,
        ptr,
        Layout::from_size_align_unchecked(count, 1),
    )
}

unsafe fn alloc_layout(layout: Layout) -> *mut u8 {
    GlobalAlloc::alloc(&GLOBAL_ALLOC, layout)
}

unsafe fn dealloc_layout(ptr: *mut u8, layout: Layout) {
    GlobalAlloc::dealloc(&GLOBAL_ALLOC, ptr, layout);
}

pub unsafe fn alloc<T>(val: T) -> *mut T {
    let layout = Layout::new::<T>();
    let ptr = alloc_layout(layout) as *mut T;
    ptr.write(val);
    ptr
}

pub unsafe fn dealloc<T>(ptr: *const T) {
    let layout = Layout::new::<T>();
    dealloc_layout(ptr as *mut u8, layout)
}

pub struct HeapBaseAddress(u64);

pub unsafe fn init() {
    let ptr = hhdm::to_virt(pmm::alloc_pages(PAGE_COUNT));
    VIRTUAL_BASE = ptr as u64;
    GLOBAL_ALLOC
        .push_region(VIRTUAL_BASE, PAGE_COUNT * PAGE_SIZE)
        .expect("unable to push region into global heap");
}
