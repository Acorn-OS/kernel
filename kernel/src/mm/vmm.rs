use crate::arch::vm::PageMap;
use crate::mm::pmm;

pub const PAGE_SIZE: usize = pmm::PAGE_SIZE;
pub const PAGE_COUNT: usize = 4096;

type BitMap = bitmap::BitMap<{ pmm::PAGE_EXP }, PAGE_COUNT>;

pub struct VirtualMemory {
    root_map: *mut PageMap,
    bitmap: BitMap,
}

impl VirtualMemory {
    pub unsafe fn alloc_pages(_pages: usize) -> *mut u8 {
        todo!()
    }

    pub unsafe fn free_pages(_ptr: *mut u8, _pages: usize) {
        todo!()
    }
}

pub fn new_kernel() -> VirtualMemory {
    todo!()
}

pub fn new_userland() -> VirtualMemory {
    todo!()
}
