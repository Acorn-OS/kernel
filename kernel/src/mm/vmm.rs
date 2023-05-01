use super::pmm::Page;
use crate::arch::vm::{self, PageMapPtr};
use crate::boot;
use crate::mm::pmm;
use alloc::alloc::Global;
use core::fmt::Debug;

pub const PAGE_SIZE: usize = pmm::PAGE_SIZE;

type AllocatorTy = freelist::FreeList<Global>;

pub struct VirtualMemory {
    root_map: PageMapPtr,
    allocator: AllocatorTy,
}

impl Debug for VirtualMemory {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VirtualMemory")
            .field("root_map", &self.root_map)
            .field("allocator", &self.allocator)
            .finish()
    }
}

impl VirtualMemory {
    pub const PAGE_SIZE: usize = PAGE_SIZE;

    pub fn alloc_pages(&mut self, pages: usize) -> *mut Page {
        let virt = self
            .allocator
            .alloc_aligned_bytes(Self::PAGE_SIZE, pages * Self::PAGE_SIZE)
            .expect("failed to allocate virtual memory");
        unsafe { vm::resv_pages(self.root_map, virt as u64, pages) };
        virt as *mut _
    }

    pub fn alloc_pages_with_base_virt(&mut self, pages: usize, virt: u64) {
        self.allocator
            .reserve_bytes(virt, pages * PAGE_SIZE)
            .expect("failed to reserve virtual memory");
        unsafe { vm::resv_pages(self.root_map, virt, pages) };
    }

    pub fn map_pages(&mut self, pages: usize, phys: u64) -> *mut Page {
        let virt = self
            .allocator
            .alloc_aligned_bytes(Self::PAGE_SIZE, pages * Self::PAGE_SIZE)
            .expect("failed to allocate virtual memory");
        unsafe { vm::alloc_pages(self.root_map, virt as u64, pages, phys) };
        virt as *mut _
    }

    pub unsafe fn map_pages_raw(&mut self, pages: usize, virt: u64, phys: u64) {
        self.allocator
            .reserve_bytes(virt, pages * Self::PAGE_SIZE)
            .expect("failed to reserve virtual memory");
        vm::alloc_pages(self.root_map, virt, pages, phys);
    }

    pub fn free_pages(_ptr: *mut u8, _pages: usize) {
        unimplemented!()
    }

    pub unsafe fn install(&self) {
        vm::install(self.root_map)
    }

    pub fn new_kernel() -> Self {
        let mut map = VirtualMemory {
            allocator: unsafe { AllocatorTy::with_allocator(Global) },
            root_map: vm::new_page_map(),
        };
        // Push allocatable region.
        map.allocator
            .push_region(0xffff800000000000, 1 << 47)
            .expect("failed to push map allocator region");
        // Memory map physical memory as HHDM.
        unsafe { vm::alloc_large_pages(map.root_map, pmm::hhdm_base(), 4, 0) };
        map.allocator
            .reserve_bytes(pmm::hhdm_base(), 4 * (1 << 30))
            .expect("failed to reserve bytes for kernel vmm");
        // Map kernel to high address.
        let s4kib = 4 << 10;
        let kernel_large_page_count = 2;
        let kernel_page_count = 4096 * 4; //(kernel_large_page_count << 30) / s4kib;
        let kernel_phys_adr = unsafe { boot::info().kernel_address.physical_base };
        let kernel_virt_adr = 0xffffffff80000000;
        unsafe { map.map_pages_raw(kernel_page_count, kernel_virt_adr, kernel_phys_adr) };
        map
    }

    pub fn new_userland() -> Self {
        let mut map = new_kernel();
        //Push allocatable region.
        let start = 1 << 20;
        let len = (1 << 47) - start;
        map.allocator
            .push_region(start, len as usize)
            .expect("failed to push map allocator region");
        map
    }
}

pub fn new_kernel() -> VirtualMemory {
    VirtualMemory::new_kernel()
}

pub fn new_userland() -> VirtualMemory {
    VirtualMemory::new_userland()
}
