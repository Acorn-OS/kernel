use super::pmm;
use core::alloc::Layout;

pub const PAGE_SIZE: usize = pmm::PAGE_SIZE;
pub const PAGE_COUNT: usize = 256;
pub const VIRT_ADR: usize = 0xfffff80000000000;
pub static mut PHYS_ADR: usize = 0;

type BitMap = bitmap::BitMap<{ pmm::PAGE_EXP }, { PAGE_COUNT.div_ceil(8) }>;

static mut BITMAP: BitMap = BitMap::new();

/// Allocate `count` amount of pages in physical memory.
pub unsafe fn alloc_pages(count: usize) -> *mut u8 {
    if let Some(index) = BITMAP.get_first_empty(count) {
        for i in 0..count {
            BITMAP.alloc(index + i);
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
pub unsafe fn dealloc_pages(_ptr: *const u8, _count: usize) {
    unimplemented!()
}

#[inline(always)]
pub unsafe fn alloc<T>(val: T) -> *mut T {
    let layout = Layout::new::<T>();
    let size = layout.size() + layout.align();
    let ptr: *mut T = alloc_pages(size.div_ceil(PAGE_SIZE)) as *mut _;
    let ptr = ptr.add(PAGE_SIZE % layout.align());
    ptr.write(val);
    ptr
}

pub unsafe fn alloc_zeroed<T>() -> *mut T {
    let layout = Layout::new::<T>();
    let size = layout.size() + layout.align();
    let ptr: *mut T = alloc_pages_zeroed(size.div_ceil(PAGE_SIZE)) as *mut _;
    ptr.add(PAGE_SIZE % layout.align())
}

#[allow(unused)]
unsafe fn dealloc<T>(ptr: *const T) {
    let layout = Layout::new::<T>();
    let size = layout.size() + (layout.align() % PAGE_SIZE != 0) as usize;
    dealloc_pages(ptr as *const u8, size)
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
