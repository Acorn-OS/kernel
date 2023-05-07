use crate::arch::vm::{self, PageMapPtr};
use core::fmt::Debug;

pub const PAGE_SIZE: usize = vm::PAGE_SIZE;

type AllocatorTy = freelist::FreeList;

pub struct VirtualMemory {
    root_map: PageMapPtr,
    allocator: AllocatorTy,
}

impl Debug for VirtualMemory {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VirtualMemory")
            .field("root_map", &self.root_map)
            .finish()
    }
}

pub enum Flags {
    Phys { flags: vm::Flags, phys: u64 },
}

impl VirtualMemory {
    pub const PAGE_SIZE: usize = PAGE_SIZE;

    pub fn map(&mut self, virt: Option<u64>, pages: usize, flags: Flags) -> u64 {
        let page_size = match flags {
            Flags::Phys { flags, .. } => {
                if flags.has(vm::Flags::SIZE_LARGE) {
                    panic!("no support for large pages")
                    //vm::LARGE_PAGE_SIZE
                } else if flags.has(vm::Flags::SIZE_MEDIUM) {
                    panic!("no support for medium pages")
                    //vm::MEDIUM_PAGE_SIZE
                } else {
                    vm::PAGE_SIZE
                }
            }
        };
        let allocated_size = page_size * pages;
        let virt = if let Some(virt) = virt {
            debug_assert!(
                virt & (page_size - 1) as u64 == 0,
                "unaligned virtual address"
            );
            self.allocator
                .reserve_bytes(virt, allocated_size)
                .expect("failed to reserve bytes");
            virt
        } else {
            self.allocator
                .alloc_aligned_bytes(page_size, allocated_size)
                .expect("failed to alloc aligned bytes") as u64
        };
        match flags {
            Flags::Phys { flags, phys } => {
                let phys = phys & !(page_size - 1) as u64;
                unsafe { vm::map(self.root_map, virt, pages, phys, flags) };
            }
        }
        virt
    }

    pub unsafe fn unmap(&mut self, _ptr: *mut u8, _pages: usize) {}

    pub unsafe fn install(&self) {
        vm::install(self.root_map)
    }

    pub fn new_userland() -> Self {
        let mut allocator = AllocatorTy::new();
        let allocator_start = 1 << 20;
        let allocator_len = (1 << 47) - allocator_start;
        allocator
            .push_region(1 << 20, allocator_len)
            .expect("failed to push region for allocator");
        let map = VirtualMemory {
            root_map: unsafe { vm::new_userland_page_map() },
            allocator,
        };
        map
    }
}

pub fn new_userland() -> VirtualMemory {
    VirtualMemory::new_userland()
}
