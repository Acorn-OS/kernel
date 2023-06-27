use super::heap::primordial::PrimordialAlloc;
use super::pmm;
use crate::arch::vm::{self, PageMapPtr, LARGE_PAGE_SIZE};
use crate::boot::BootInfo;
use crate::symbols;
use crate::util::adr::{PhysAdr, VirtAdr};
use core::fmt::Debug;

pub const PAGE_SIZE: usize = vm::PAGE_SIZE;

type AllocatorTy = allocators::freelist::FreeList<PrimordialAlloc>;

pub type Flags = vm::VMFlags;

pub struct VMM {
    root_map: PageMapPtr,
    allocator: AllocatorTy,
}

impl Debug for VMM {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VirtualMemory")
            .field("root_map", &self.root_map)
            .finish()
    }
}

impl VMM {
    pub const PAGE_SIZE: usize = PAGE_SIZE;

    /// if `virt` is null, map will allocate new memory.
    pub fn map(
        &mut self,
        virt: Option<VirtAdr>,
        pages: usize,
        flags: Flags,
        phys_adr: PhysAdr,
    ) -> VirtAdr {
        let page_size = if flags.has(Flags::SIZE_LARGE) {
            vm::LARGE_PAGE_SIZE
        } else if flags.has(Flags::SIZE_MEDIUM) {
            vm::MEDIUM_PAGE_SIZE
        } else {
            vm::PAGE_SIZE
        };
        let allocated_size = page_size * pages;
        let virt = if let Some(virt) = virt {
            debug_assert!(virt.is_aligned(page_size), "unaligned virtual address");
            let virt = virt.align_floor(page_size);
            self.allocator
                .reserve_bytes(virt.adr(), allocated_size)
                .expect("failed to reserve bytes");
            virt
        } else {
            VirtAdr::new(
                self.allocator
                    .alloc_aligned_bytes(page_size, allocated_size)
                    .expect("failed to alloc aligned bytes") as _,
            )
        };
        debug_assert!(phys_adr.is_aligned(page_size));
        let phys_adr = phys_adr.align_floor(page_size);
        unsafe { vm::map(self.root_map, virt, pages, phys_adr, flags) };
        virt
    }

    /// if `vadr` is null, then take an empty range.
    pub fn reserve_range(
        &mut self,
        vadr: Option<VirtAdr>,
        pages: usize,
    ) -> Result<VirtAdr, allocators::freelist::Error> {
        if let Some(vadr) = vadr {
            self.allocator
                .reserve_bytes(vadr.adr(), pages * PAGE_SIZE)
                .map(|_| vadr)
        } else {
            self.allocator
                .alloc_aligned_bytes(PAGE_SIZE, pages * PAGE_SIZE)
                .map(|ptr| VirtAdr::new(ptr as u64))
        }
    }

    pub fn contains_page(&self, virt: VirtAdr) -> bool {
        self.virt_to_phys(virt).is_some()
    }

    pub fn virt_to_phys(&self, virt: VirtAdr) -> Option<PhysAdr> {
        virt.to_phys(self.get_page_map())
    }

    pub unsafe fn unmap(&mut self, _ptr: *mut u8, _pages: usize) {}

    pub unsafe fn install(&self) {
        vm::install(self.root_map)
    }

    pub fn get_page_map(&self) -> PageMapPtr {
        self.root_map
    }

    pub fn new_userland() -> Self {
        let mut allocator = unsafe { AllocatorTy::with_allocator(PrimordialAlloc) };
        let allocator_start = 1u64 << 20;
        let allocator_len = (1 << 47) - PAGE_SIZE - allocator_start as usize;
        allocator
            .push_region(allocator_start, allocator_len)
            .expect("failed to push region for allocator");
        VMM {
            root_map: unsafe { vm::new_userland_page_map() },
            allocator,
        }
    }
}

pub fn new_userland() -> VMM {
    VMM::new_userland()
}

pub unsafe fn create_kernel(boot_info: &BootInfo) -> VMM {
    let mut vmm = VMM {
        root_map: vm::kernel_page_map(),
        allocator: AllocatorTy::with_allocator(PrimordialAlloc),
    };
    // Memory map physical memory as HHDM.
    let hhdm_pages = pmm::hhdm_len() / LARGE_PAGE_SIZE;
    vmm.root_map.map(
        VirtAdr::new(pmm::hhdm_base()),
        hhdm_pages,
        PhysAdr::new(0),
        Flags::PRESENT | Flags::RW | Flags::SIZE_LARGE,
    );
    // create mapable region
    let region_start = pmm::hhdm_base() + pmm::hhdm_len() as u64;
    let region_end = u64::MAX - 2 * LARGE_PAGE_SIZE as u64 + 1;
    let region_len = region_end - region_start;
    vmm.allocator
        .push_region(region_start, region_len as usize)
        .expect("failed to push region");
    // Map kernel text section.
    let mut kernel_phys_adr = PhysAdr::new(boot_info.kernel_address.physical_base);
    let section_text_start = align_floor!(symbols::section_text_start(), PAGE_SIZE as u64);
    let section_text_len = symbols::section_text_end() - section_text_start;
    let section_text_pages = pages!(section_text_len) as u64;
    vmm.root_map.map(
        VirtAdr::new(section_text_start),
        section_text_pages as usize,
        kernel_phys_adr,
        vm::VMFlags::PRESENT,
    );
    kernel_phys_adr = kernel_phys_adr.add(section_text_pages as usize * PAGE_SIZE);
    // Map kernel read only section.
    let section_r_start = align_floor!(symbols::section_r_start(), PAGE_SIZE as u64);
    let section_r_len = symbols::section_r_end() - section_r_start;
    let section_r_pages = pages!(section_r_len) as u64;
    vmm.root_map.map(
        VirtAdr::new(section_r_start),
        section_r_pages as usize,
        kernel_phys_adr,
        vm::VMFlags::PRESENT | vm::VMFlags::XD,
    );
    kernel_phys_adr = kernel_phys_adr.add(section_r_pages as usize * PAGE_SIZE);
    // Map kernel read and write section.
    let section_rw_start = align_floor!(symbols::section_rw_start(), PAGE_SIZE as u64);
    let section_rw_len = symbols::section_rw_end() - section_rw_start;
    let section_rw_pages = pages!(section_rw_len);
    vmm.root_map.map(
        VirtAdr::new(section_rw_start),
        section_rw_pages as usize,
        kernel_phys_adr,
        vm::VMFlags::PRESENT | vm::VMFlags::RW | vm::VMFlags::XD,
    );
    vmm
}
