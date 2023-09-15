pub mod error;

use super::pmm;
use crate::arch::vadr;
use crate::arch::vm::{self, PageMapPtr, VMFlags};
use crate::boot::BootInfo;
use crate::symbols;
use crate::util::adr::{PhysAdr, VirtAdr};
use alloc::collections::BTreeMap;
use allocators::freelist::FreeList;
use core::fmt::Debug;
use error::Result;

pub const PAGE_SIZE: usize = vm::PAGE_SIZE;
pub const MEDIUM_PAGE_SIZE: usize = vm::MEDIUM_PAGE_SIZE;
pub const LARGE_PAGE_SIZE: usize = vm::LARGE_PAGE_SIZE;

type AllocatorTy = FreeList;

enum RegionType {
    Reserved,
    Normal,
}

struct Region {
    page_cnt: usize,
    ty: RegionType,
}

pub struct VMM {
    root_map: PageMapPtr,
    regions: BTreeMap<vadr, Region>,
    free_regions: AllocatorTy,
}

bit_flags!(
    pub struct Flags(u64);
    EXECUTABLE = 0;
    USER = 1;
    LARGE_PAGE_SIZE = 2;
    MEDIUM_PAGE_SIZE = 3;
    RW = 4;
);

pub enum MapTy {
    None,
    Phys { adr: PhysAdr },
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
        ty: MapTy,
    ) -> Result<VirtAdr> {
        let page_size = Self::page_size_from_flags(flags);
        let virt = if let Some(virt) = virt {
            self.alloc_reserve_range(virt, page_size, pages).unwrap();
            virt
        } else {
            self.alloc_new(page_size, pages)?
        };
        let mut vm_flags = VMFlags::NONE;
        macro_rules! sf {
            ($f:expr => $f2:expr) => {
                if flags.has($f) {
                    vm_flags |= $f2
                }
            };
            (not $f:expr => $f2:expr) => {
                if !flags.has($f) {
                    vm_flags |= $f2
                }
            };
        }
        sf!(not Flags::EXECUTABLE => VMFlags::XD);
        sf!(Flags::LARGE_PAGE_SIZE => VMFlags::SIZE_LARGE);
        sf!(Flags::MEDIUM_PAGE_SIZE => VMFlags::SIZE_MEDIUM);
        sf!(Flags::RW => VMFlags::RW);
        sf!(Flags::USER => VMFlags::USER);

        let phys_adr = match ty {
            MapTy::None => PhysAdr::null(),
            MapTy::Phys { adr } => {
                vm_flags |= VMFlags::PRESENT;
                debug_assert!(adr.is_aligned(page_size));
                adr.align_floor(page_size)
            }
        };
        debug!("virt: {:016x}", virt.adr());
        debug!("{:?}", self.free_regions);
        unsafe { vm::map(self.root_map, virt, pages, phys_adr, vm_flags) };
        Ok(virt)
    }

    pub unsafe fn unmap(&mut self, _ptr: *mut u8, _pages: usize) -> Result<()> {
        unimplemented!()
    }

    pub fn contains_page(&self, virt: VirtAdr) -> bool {
        self.virt_to_phys(virt).is_some()
    }

    pub fn virt_to_phys(&self, virt: VirtAdr) -> Option<PhysAdr> {
        virt.to_phys(self.get_page_map())
    }

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
        VMM {
            root_map: unsafe { vm::new_userland_page_map() },
            regions: BTreeMap::new(),
            free_regions: allocator,
        }
    }

    fn page_size_from_flags(flags: Flags) -> usize {
        if flags.has(Flags::LARGE_PAGE_SIZE) {
            LARGE_PAGE_SIZE
        } else if flags.has(Flags::MEDIUM_PAGE_SIZE) {
            MEDIUM_PAGE_SIZE
        } else {
            PAGE_SIZE
        }
    }

    /// if `vadr` is null, then take an empty range.
    fn alloc_reserve_range(&mut self, vadr: VirtAdr, page_size: usize, pages: usize) -> Result<()> {
        Ok(self
            .free_regions
            .reserve_bytes(vadr.adr(), pages * page_size)
            .map(|_| ())?)
    }

    fn alloc_new(&mut self, page_size: usize, pages: usize) -> Result<VirtAdr> {
        Ok(self
            .free_regions
            .alloc_aligned_bytes(page_size, pages * page_size)
            .map(|adr| VirtAdr::new(adr as u64))?)
    }

    fn alloc_free(&mut self, vadr: VirtAdr, page_size: usize, pages: usize) -> Result<()> {
        Ok(unsafe {
            self.free_regions
                .free_bytes(vadr.ptr(), page_size * pages)?
        })
    }
}

pub fn new_userland() -> VMM {
    VMM::new_userland()
}

static mut INIT_KERNEL_VMM: bool = false;
pub unsafe fn init_kernel_vmm(boot_info: &BootInfo) -> VMM {
    debug_assert!(!INIT_KERNEL_VMM);
    INIT_KERNEL_VMM = true;

    let mut allocator = AllocatorTy::new();
    let allocator_start = 0x1ffff << 47;
    let allocator_len = 1 << 47;
    allocator
        .push_region(allocator_start, allocator_len)
        .expect("failed to push region for allocator");
    let mut vmm = VMM {
        root_map: vm::kernel_page_map(),
        regions: BTreeMap::new(),
        free_regions: allocator,
    };
    // Memory map physical memory as HHDM.
    let hhdm_pages = pmm::hhdm_len() / LARGE_PAGE_SIZE;
    vmm.alloc_reserve_range(pmm::hhdm_base(), LARGE_PAGE_SIZE, 512 * 2)
        .unwrap();
    vmm.map(
        Some(pmm::hhdm_base()),
        hhdm_pages,
        Flags::RW | Flags::LARGE_PAGE_SIZE,
        MapTy::Phys {
            adr: PhysAdr::new(0),
        },
    )
    .unwrap();
    // Reserve allocation from the top 2GiB.
    vmm.alloc_reserve_range(VirtAdr::new(0xffffffff80000000), PAGE_SIZE, 524288)
        .unwrap();
    // Map kernel text section.
    let mut kernel_phys_adr = PhysAdr::new(boot_info.kernel_address.physical_base);
    let section_text_start = align_floor!(symbols::section_text_start(), PAGE_SIZE as u64);
    let section_text_len = symbols::section_text_end() - section_text_start;
    let section_text_pages = pages!(section_text_len) as u64;
    vmm.map(
        Some(VirtAdr::new(section_text_start)),
        section_text_pages as usize,
        Flags::EXECUTABLE,
        MapTy::Phys {
            adr: kernel_phys_adr,
        },
    )
    .unwrap();
    kernel_phys_adr = kernel_phys_adr.add(section_text_pages as usize * PAGE_SIZE);
    // Map kernel read only section.
    let section_r_start = align_floor!(symbols::section_r_start(), PAGE_SIZE as u64);
    let section_r_len = symbols::section_r_end() - section_r_start;
    let section_r_pages = pages!(section_r_len) as u64;
    vmm.map(
        Some(VirtAdr::new(section_r_start)),
        section_r_pages as usize,
        Flags::NONE,
        MapTy::Phys {
            adr: kernel_phys_adr,
        },
    )
    .unwrap();
    kernel_phys_adr = kernel_phys_adr.add(section_r_pages as usize * PAGE_SIZE);
    // Map kernel read and write section.
    let section_rw_start = align_floor!(symbols::section_rw_start(), PAGE_SIZE as u64);
    let section_rw_len = symbols::section_rw_end() - section_rw_start;
    let section_rw_pages = pages!(section_rw_len);
    vmm.map(
        Some(VirtAdr::new(section_rw_start)),
        section_rw_pages as usize,
        Flags::RW,
        MapTy::Phys {
            adr: kernel_phys_adr,
        },
    )
    .unwrap();
    vmm
}
