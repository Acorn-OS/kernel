use super::fb;
use crate::boot::limine;
use crate::mm::{heap, pmm};
use core::arch::asm;
use core::mem::size_of;

assert_eq_size!(usize, u64);

pub const SMALL_PAGE_SIZE: u64 = 1 << 12;
pub const MEDIUM_PAGE_SIZE: u64 = 1 << 21;
pub const LARGE_PAGE_SIZE: u64 = 1 << 30;

bitfield! {
    #[derive(Clone, Copy)]
    #[repr(C, packed)]
    struct PageMapEntry(u64) {
        pub p :bool @ 0,
        pub rw: bool @ 1,
        pub us: bool @ 2,
        pub pwt: bool @ 3,
        pub pcd: bool @ 4,
        pub a: bool @ 5,
        pub ps: bool @ 7,
        inner_adr: u64 @ 12..=51,
        pub xd: bool @ 63,
    }
}
const_assert!(size_of::<PageMapEntry>() == 8);

impl PageMapEntry {
    pub fn adr(&self) -> u64 {
        ((((self.inner_adr() << 12) as i64) << 16) >> 16) as u64
    }

    pub fn set_adr(&mut self, adr: u64) {
        self.set_inner_adr(adr >> 12)
    }
}

#[repr(C, align(4096))]
pub struct PageMap {
    entries: [PageMapEntry; 512],
}
const_assert!(size_of::<PageMap>() == 4096);

mod get {
    use super::*;

    pub unsafe fn allocate(page_map: *mut PageMap, index: usize) -> *mut PageMap {
        debug_assert!(index < 512);
        let index = index & 0x1FF;
        if !(*page_map).entries[index].p() {
            let new: *mut PageMap =
                pmm::alloc_pages_zeroed(size_of::<PageMap>().div_ceil(pmm::PAGE_SIZE)) as *mut _;
            let page_entry = &mut (*page_map).entries[index];
            page_entry.set_adr(new as u64);
            page_entry.set_p(true);
            page_entry.set_rw(true);
            new
        } else {
            (*page_map).entries[index].adr() as *mut PageMap
        }
    }
}

#[derive(Clone, Copy)]
#[repr(u64)]
pub enum AllocSize {
    SmallPage = SMALL_PAGE_SIZE,
    MediumPage = MEDIUM_PAGE_SIZE,
    LagePage = LARGE_PAGE_SIZE,
}

impl AllocSize {
    pub fn size(&self) -> u64 {
        *self as u64
    }
}

unsafe fn set(page_map: *mut PageMap, index: usize, adr: u64, large: bool) {
    debug_assert!(index < 512);
    let index = index & 0x1FF;
    let entry = &mut (*page_map).entries[index];
    entry.set_adr(adr);
    entry.set_ps(large);
    entry.set_p(true);
    entry.set_rw(true);
}

unsafe fn resv(page_map: *mut PageMap, index: usize, adr: u64, large: bool) -> Result<(), ()> {
    debug_assert!(index < 512);
    let index = index & 0x1FF;
    let entry = &mut (*page_map).entries[index];
    if entry.p() {
        Err(())
    } else {
        entry.set_adr(adr);
        entry.set_ps(large);
        entry.set_p(true);
        entry.set_rw(true);
        Ok(())
    }
}

impl PageMap {
    pub const fn new() -> Self {
        Self {
            entries: [PageMapEntry(0); 512],
        }
    }

    pub unsafe fn install(&mut self) {
        PageMap::install_ptr(self as *const _);
    }

    unsafe fn install_ptr(ptr: *const Self) {
        debug!("installing new page table {ptr:?}");
        unsafe {
            asm!(
                "mov cr3, rax",
                in("rax") ptr as u64,
                options(nostack)
            );
        }
    }

    #[inline]
    fn divide_virt_adr(virt: u64) -> (usize, usize, usize, usize, u64) {
        (
            (virt as usize >> 39) & 0x1FF,
            (virt as usize >> 30) & 0x1FF,
            (virt as usize >> 21) & 0x1FF,
            (virt as usize >> 12) & 0x1FF,
            virt & 0xFFF,
        )
    }

    /// reserves virtual pages in memory.
    /// Gives an error if the page was already reserved.
    pub unsafe fn resv(
        &mut self,
        size: AllocSize,
        mut virt: u64,
        pages: usize,
        mut phys: u64,
    ) -> Result<(), ()> {
        match size {
            AllocSize::SmallPage => {
                for _ in 0..pages {
                    let (d0, d1, d2, d3, _) = Self::divide_virt_adr(virt);
                    self.resv_map4((d0, d1, d2, d3), phys & !0xFFF)?;
                    virt += size.size();
                    phys += size.size();
                }
            }
            AllocSize::MediumPage => {
                for _ in 0..pages {
                    let (d0, d1, d2, _, _) = Self::divide_virt_adr(virt);
                    self.resv_map3((d0, d1, d2), phys & !((1 << 21) - 1))?;
                    virt += size.size();
                    phys += size.size();
                }
            }
            AllocSize::LagePage => {
                for _ in 0..pages {
                    let (d0, d1, _, _, _) = Self::divide_virt_adr(virt);
                    self.resv_map2((d0, d1), phys & !((1 << 30) - 1))?;
                    virt += size.size();
                    phys += size.size();
                }
            }
        }
        Ok(())
    }

    unsafe fn resv_map2(&mut self, index: (usize, usize), adr: u64) -> Result<(), ()> {
        let ptr = self.as_mut_ptr();
        let (d0, d1) = index;
        resv(get::allocate(ptr, d0), d1, adr, true)
    }

    unsafe fn resv_map3(&mut self, index: (usize, usize, usize), adr: u64) -> Result<(), ()> {
        let ptr = self.as_mut_ptr();
        let (d0, d1, d2) = index;
        resv(get::allocate(get::allocate(ptr, d0), d1), d2, adr, true)
    }

    unsafe fn resv_map4(
        &mut self,
        index: (usize, usize, usize, usize),
        adr: u64,
    ) -> Result<(), ()> {
        let ptr = self.as_mut_ptr();
        let (d0, d1, d3, d4) = index;
        resv(
            get::allocate(get::allocate(get::allocate(ptr, d0), d1), d3),
            d4,
            adr,
            false,
        )
    }

    pub unsafe fn alloc(&mut self, size: AllocSize, mut virt: u64, pages: usize, mut phys: u64) {
        match size {
            AllocSize::SmallPage => {
                for _ in 0..pages {
                    let (d0, d1, d2, d3, _) = Self::divide_virt_adr(virt);
                    self.alloc_map4((d0, d1, d2, d3), phys & !0xFFF);
                    virt += size.size();
                    phys += size.size();
                }
            }
            AllocSize::MediumPage => {
                for _ in 0..pages {
                    let (d0, d1, d2, _, _) = Self::divide_virt_adr(virt);
                    self.alloc_map3((d0, d1, d2), phys & !((1 << 21) - 1));
                    virt += size.size();
                    phys += size.size();
                }
            }
            AllocSize::LagePage => {
                for _ in 0..pages {
                    let (d0, d1, _, _, _) = Self::divide_virt_adr(virt);
                    self.alloc_map2((d0, d1), phys & !((1 << 30) - 1));
                    virt += size.size();
                    phys += size.size();
                }
            }
        }
    }

    unsafe fn alloc_map2(&mut self, index: (usize, usize), adr: u64) {
        let ptr = self.as_mut_ptr();
        let (d0, d1) = index;
        set(get::allocate(ptr, d0), d1, adr, true);
    }

    unsafe fn alloc_map3(&mut self, index: (usize, usize, usize), adr: u64) {
        let ptr = self.as_mut_ptr();
        let (d0, d1, d2) = index;
        set(get::allocate(get::allocate(ptr, d0), d1), d2, adr, true);
    }

    unsafe fn alloc_map4(&mut self, index: (usize, usize, usize, usize), adr: u64) {
        let ptr = self.as_mut_ptr();
        let (d0, d1, d3, d4) = index;
        set(
            get::allocate(get::allocate(get::allocate(ptr, d0), d1), d3),
            d4,
            adr,
            false,
        );
    }

    fn as_mut_ptr(&self) -> *mut PageMap {
        self as *const _ as *mut PageMap
    }
}

pub unsafe fn new_kernel() -> Result<*mut PageMap, *mut PageMap> {
    let map = pmm::alloc_pages_zeroed(1) as *mut PageMap;
    let s4kib = 4 << 10;
    // Identity map the initial 4GiB of physical memory,
    (*map).resv(AllocSize::LagePage, 0, 4, 0).or(Err(map))?;
    // Map heap memory.
    (*map)
        .resv(
            AllocSize::SmallPage,
            heap::VIRT_ADR as u64,
            heap::PAGE_COUNT,
            heap::PHYS_ADR as u64,
        )
        .or(Err(map))?;
    // Map video display memory
    (*map)
        .resv(
            AllocSize::SmallPage,
            fb::VIRT_ADR as u64,
            fb::PAGE_COUNT,
            fb::PHYS_ADR as u64,
        )
        .or(Err(map))?;
    // Map kernel to high address.
    let page_count = (30 << 20) / s4kib;
    let phys_adr = limine::kernel_address().physical_base;
    let virt_adr = 0xffffffff80000000 as u64;
    (*map).alloc(AllocSize::SmallPage, virt_adr, page_count, phys_adr);
    Ok(map)
}

pub unsafe fn install(map: *mut PageMap) {
    PageMap::install_ptr(map)
}
