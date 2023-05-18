use crate::arch::vm::{self, PageMapPtr};
use crate::boot::BootInfo;
use crate::symbols;
use core::fmt::Debug;
use spin::Mutex;

use super::pmm;

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
                is_aligned!(virt, page_size as u64),
                "unaligned virtual address"
            );
            let virt = align_floor!(virt, page_size as u64);
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
                debug_assert!(is_aligned!(phys, page_size as u64));
                let phys = align_floor!(phys, page_size as u64);
                unsafe { vm::map(self.root_map, virt, pages, phys, flags) };
            }
        }
        virt
    }

    pub fn contains_page(&self, virt: u64) -> bool {
        self.virt_to_phys(virt).is_some()
    }

    pub fn virt_to_phys(&self, virt: u64) -> Option<u64> {
        if let Some(entry) = unsafe { vm::get_page_entry(self.root_map, virt) } {
            let entry = unsafe { entry.as_ref() };
            if entry.present() {
                Some(entry.adr())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub unsafe fn unmap(&mut self, _ptr: *mut u8, _pages: usize) {}

    pub unsafe fn install(&self) {
        vm::install(self.root_map)
    }

    pub fn get_page_map(&self) -> PageMapPtr {
        self.root_map
    }

    pub fn new_userland() -> Self {
        let mut allocator = AllocatorTy::new();
        let allocator_start = 1u64 << 20;
        let allocator_len = (1 << 47) - PAGE_SIZE - allocator_start as usize;
        allocator
            .push_region(allocator_start, allocator_len)
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

struct KernelVmm(VirtualMemory);

unsafe impl Sync for KernelVmm {}
unsafe impl Send for KernelVmm {}

static KERNEL_VMM: Mutex<KernelVmm> = Mutex::new(KernelVmm(VirtualMemory {
    root_map: unsafe { PageMapPtr::nullptr() },
    allocator: AllocatorTy::new(),
}));

pub unsafe fn init(boot_info: &BootInfo) {
    // Memory map physical memory as HHDM.
    let mut kernel_vmm = KERNEL_VMM.lock();
    kernel_vmm.0.root_map = vm::kernel_page_map();
    let kernel_vmm = &mut kernel_vmm.0;
    let region_start = (0xffff << 48) | (1 << 47);
    let region_len = 1 << 47;
    kernel_vmm
        .allocator
        .push_region(region_start, region_len)
        .expect("failed to push region");
    kernel_vmm.root_map.map(
        pmm::hhdm_base(),
        4,
        0,
        vm::Flags::PRESENT | vm::Flags::RW | vm::Flags::SIZE_LARGE,
    );
    kernel_vmm
        .allocator
        .reserve_bytes(pmm::hhdm_base(), 4 * vm::LARGE_PAGE_SIZE)
        .expect("failed to reserve memory");
    // Map kernel text section.
    let mut kernel_phys_adr = boot_info.kernel_address.physical_base;
    let section_text_start = align_floor!(symbols::section_text_start(), PAGE_SIZE as u64);
    let section_text_len = symbols::section_text_end() - section_text_start;
    let section_text_pages = pages!(section_text_len) as u64;
    kernel_vmm.map(
        Some(section_text_start),
        section_text_pages as usize,
        Flags::Phys {
            flags: vm::Flags::PRESENT,
            phys: kernel_phys_adr,
        },
    );
    kernel_phys_adr += section_text_pages * PAGE_SIZE as u64;
    // Map kernel read only section.
    let section_r_start = align_floor!(symbols::section_r_start(), PAGE_SIZE as u64);
    let section_r_len = symbols::section_r_end() - section_r_start;
    let section_r_pages = pages!(section_r_len) as u64;
    kernel_vmm.map(
        Some(section_r_start),
        section_r_pages as usize,
        Flags::Phys {
            flags: vm::Flags::PRESENT | vm::Flags::XD,
            phys: kernel_phys_adr,
        },
    );
    kernel_phys_adr += section_r_pages * PAGE_SIZE as u64;
    // Map kernel read and write section.
    let section_rw_start = align_floor!(symbols::section_rw_start(), PAGE_SIZE as u64);
    let section_rw_len = symbols::section_rw_end() - section_rw_start;
    let section_rw_pages = pages!(section_rw_len);
    kernel_vmm.map(
        Some(section_rw_start),
        section_rw_pages as usize,
        Flags::Phys {
            flags: vm::Flags::PRESENT | vm::Flags::RW | vm::Flags::XD,
            phys: kernel_phys_adr,
        },
    );
    kernel_vmm.install();
}
