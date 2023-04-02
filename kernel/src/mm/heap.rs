use super::pmm;
use core::alloc::{Allocator, GlobalAlloc, Layout};

pub const PAGE_SIZE: usize = pmm::PAGE_SIZE;
const PAGE_EXP: usize = pmm::PAGE_EXP;
pub const PAGE_COUNT: usize = 256;
pub const VIRT_ADR: u64 = 0xfffff80000000000;
pub static mut PHYS_ADR: u64 = 0;

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
                pmm::alloc_pages(1),
                layout.size(),
            ))
        })
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, _layout: Layout) {
        pmm::dealloc_pages(ptr.as_ptr(), 1)
    }
}

type AllocatorTy = freelist::FreeListAllocator<NodeAllocator>;

#[global_allocator]
static GLOBAL_ALLOC: AllocatorTy = unsafe { AllocatorTy::new(NodeAllocator) };

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

pub unsafe fn init() {
    PHYS_ADR = pmm::alloc_pages(PAGE_COUNT) as u64;
    info!(
        "initializing heap at virtual address '0x{VIRT_ADR:016X}' with physical adr '0x{PHYS_ADR:016X}'"
    );
    info!("heap page size '{PAGE_SIZE}' and page count '{PAGE_COUNT}'");
    debug!(
        "heap virtual range '0x{VIRT_ADR:016X} -> 0x{:016X}'",
        VIRT_ADR + (PAGE_COUNT * PAGE_SIZE) as u64
    );
    debug!(
        "heap physical range '0x{:016X} -> 0x{:016X}'",
        PHYS_ADR,
        PHYS_ADR + (PAGE_COUNT * PAGE_SIZE) as u64
    );
    GLOBAL_ALLOC
        .push_region(VIRT_ADR, PAGE_COUNT * PAGE_SIZE)
        .expect("unable to push region into global heap");
}
