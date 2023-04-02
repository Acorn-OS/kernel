use super::pmm;
use core::alloc::{GlobalAlloc, Layout};

pub const PAGE_SIZE: usize = pmm::PAGE_SIZE;
pub const PAGE_COUNT: usize = 256;
pub const VIRT_ADR: usize = 0xfffff80000000000;
pub static mut PHYS_ADR: usize = 0;

type Allocator = bitmap::BitMap<{ pmm::PAGE_EXP }, { PAGE_COUNT.div_ceil(8) }>;

static mut ALLOCATOR: Allocator = Allocator::new();

/// Allocate `count` amount of pages in physical memory.
pub unsafe fn alloc_pages(count: usize) -> *mut u8 {
    if let Some(index) = ALLOCATOR.get_first_empty(count) {
        for i in 0..count {
            ALLOCATOR.alloc(index + i);
        }
        (VIRT_ADR + index * PAGE_SIZE) as *mut u8
    } else {
        panic!("insufficient kernel heap memory")
    }
}

/// Allocate `count` amount of zeroed pages in physical memory.
pub unsafe fn alloc_pages_zeroed(count: usize) -> *mut u8 {
    let ptr = alloc_pages(count);
    ptr.write_bytes(0, count * PAGE_SIZE);
    ptr
}

/// Deallocates `count` pages at physical address `ptr`.
pub unsafe fn dealloc_pages(ptr: *const u8, count: usize) {
    let base = ptr as usize - VIRT_ADR;
    let index = base / ALLOCATOR.page_size();
    for i in 0..count {
        ALLOCATOR.free(index + i);
    }
}

unsafe fn alloc_layout(layout: Layout) -> *mut u8 {
    let size = layout.size() + layout.align();
    let ptr: *mut u8 = alloc_pages(size.div_ceil(PAGE_SIZE)) as *mut _;
    let ptr = ptr.add(PAGE_SIZE % layout.align());
    ptr
}

unsafe fn dealloc_layout(ptr: *mut u8, layout: Layout) {
    let size = layout.size() + (layout.align() % PAGE_SIZE != 0) as usize;
    dealloc_pages(ptr as *const u8, size.div_ceil(PAGE_SIZE))
}

#[inline(always)]
pub unsafe fn alloc<T>(val: T) -> *mut T {
    let layout = Layout::new::<T>();
    let ptr = alloc_layout(layout) as *mut T;
    ptr.write(val);
    ptr
}

#[allow(unused)]
unsafe fn dealloc<T>(ptr: *const T) {
    let layout = Layout::new::<T>();
    dealloc_layout(ptr as *mut u8, layout)
}

pub unsafe fn init() {
    PHYS_ADR = pmm::alloc_pages(PAGE_COUNT) as usize;
    info!(
        "initializing heap at virtual address '0x{VIRT_ADR:016X}' with physical adr '0x{PHYS_ADR:016X}'"
    );
    info!("heap page size '{PAGE_SIZE}' and page count '{PAGE_COUNT}'");
    debug!(
        "heap virtual range '0x{VIRT_ADR:016X} -> 0x{:016X}'",
        VIRT_ADR + PAGE_COUNT * PAGE_SIZE
    );
    debug!(
        "heap physical range '0x{:016X} -> 0x{:016X}'",
        PHYS_ADR,
        PHYS_ADR + PAGE_COUNT * PAGE_SIZE
    );
}

struct HeapGlobalAlloc;

#[global_allocator]
static GLOBAL_ALLOC: HeapGlobalAlloc = HeapGlobalAlloc;

unsafe impl GlobalAlloc for HeapGlobalAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        alloc_layout(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        dealloc_layout(ptr, layout)
    }
}
