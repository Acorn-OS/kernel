use crate::mm::pmm;
use core::arch::asm;
use core::mem::size_of;

assert_eq_size!(usize, u64);

pub const SMALL_PAGE_SIZE: u64 = 1 << 12;
pub const MEDIUM_PAGE_SIZE: u64 = 1 << 21;
pub const LARGE_PAGE_SIZE: u64 = 1 << 30;

pub const PAGE_SIZE: usize = SMALL_PAGE_SIZE as usize;

bitfield! {
    #[derive(Clone, Copy)]
    #[repr(C, packed)]
    pub struct PageMapEntry(u64) {
        p :bool @ 0,
        rw: bool @ 1,
        us: bool @ 2,
        pwt: bool @ 3,
        pcd: bool @ 4,
        a: bool @ 5,
        ps: bool @ 7,
        resv: bool @ 10,
        inner_adr: u64 @ 12..=51,
        xd: bool @ 63,
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

    pub fn is_resv(&self) -> bool {
        self.resv()
    }

    pub fn set_present(&mut self) {
        self.set_p(true)
    }

    pub fn new_kernel_small(phys_adr: u64) -> Self {
        let mut entry = Self(0);
        entry.set_adr(phys_adr);
        entry.set_p(true);
        entry.set_rw(true);
        entry
    }
}

#[repr(C, align(4096))]
pub struct PageMap {
    entries: [PageMapEntry; 512],
}
const_assert!(size_of::<PageMap>() == PAGE_SIZE);
const_assert!(PAGE_SIZE == pmm::PAGE_SIZE);

mod get {
    use super::*;

    pub unsafe fn allocate(page_map: *mut PageMap, index: usize) -> *mut PageMap {
        debug_assert!(index < 512);
        let index = index & 0x1FF;
        let entry = &mut (*page_map).entries[index];
        if !entry.p() {
            let new = new_page_map();
            *entry = PageMapEntry::new_kernel_small(new as u64);
            new
        } else if entry.p() && entry.ps() {
            todo!()
        } else {
            entry.adr() as *mut PageMap
        }
    }

    pub unsafe fn get(page_map: *mut PageMap, index: usize) -> Option<*mut PageMap> {
        debug_assert!(index < 512);
        let index = index & 0x1FF;
        let entry = &(*page_map).entries[index];
        if !entry.p() {
            None
        } else if entry.p() && entry.ps() {
            todo!()
        } else {
            Some(entry.adr() as *mut PageMap)
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

unsafe fn resv(page_map: *mut PageMap, index: usize) {
    debug_assert!(index < 512);
    let index = index & 0x1FF;
    let entry = &mut (*page_map).entries[index];
    entry.set_adr(0);
    entry.set_p(false);
    entry.set_rw(true);
    entry.set_resv(true);
}

unsafe fn get(page_map: *mut PageMap, index: usize) -> *mut PageMapEntry {
    debug_assert!(index < 512);
    let index = index & 0x1FF;
    (*page_map).entries.as_mut_ptr().add(index)
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

    pub unsafe fn reserve(&mut self, mut virt: u64, pages: usize) {
        let ptr = self.as_mut_ptr();
        for _ in 0..pages {
            let (d0, d1, d2, d3, _) = Self::divide_virt_adr(virt);
            resv(
                get::allocate(get::allocate(get::allocate(ptr, d0), d1), d2),
                d3,
            );
            virt += PAGE_SIZE as u64;
        }
    }

    pub unsafe fn get(&mut self, virt: u64) -> Option<*mut PageMapEntry> {
        let ptr = self.as_mut_ptr();
        let (d0, d1, d2, d3, _) = Self::divide_virt_adr(virt);
        Some(get(get::get(get::get(get::get(ptr, d0)?, d1)?, d2)?, d3))
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
        let (d0, d1, d2, d3) = index;
        set(
            get::allocate(get::allocate(get::allocate(ptr, d0), d1), d2),
            d3,
            adr,
            false,
        );
    }

    fn as_mut_ptr(&self) -> *mut PageMap {
        self as *const _ as *mut PageMap
    }
}

pub(super) unsafe fn get_cur() -> *mut PageMap {
    let out: u64;
    asm!(
        "mov {out}, cr3",
        out = out(reg) out
    );
    out as *mut PageMap
}

pub fn new_page_map() -> *mut PageMap {
    pmm::alloc_pages_zeroed(1) as *mut PageMap
}

pub unsafe fn alloc_pages(map: *mut PageMap, virt: u64, pages: usize, phys: u64) {
    (*map).alloc(AllocSize::SmallPage, virt, pages, phys)
}

pub unsafe fn alloc_large_pages(map: *mut PageMap, virt: u64, pages: usize, phys: u64) {
    (*map).alloc(AllocSize::LagePage, virt, pages, phys)
}

pub unsafe fn free_pages(_map: *mut PageMap, _virt: u64, _count: usize) {}

pub unsafe fn install(map: *mut PageMap) {
    PageMap::install_ptr(map)
}

pub unsafe fn get_page_entry(map: *mut PageMap, virt: u64) -> Option<*mut PageMapEntry> {
    (*map).get(virt)
}

pub unsafe fn resv_pages(map: *mut PageMap, virt: u64, pages: usize) {
    (*map).reserve(virt, pages);
}
