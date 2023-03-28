use crate::boot::limine;
use crate::mm::{heap, pmm};
use core::arch::asm;
use core::mem::size_of;

use super::fb;

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

unsafe fn set(page_map: *mut PageMap, index: usize, adr: u64, large: bool) {
    debug_assert!(index < 512);
    let index = index & 0x1FF;
    let entry = &mut (*page_map).entries[index];
    entry.set_adr(adr);
    entry.set_ps(large);
    entry.set_p(true);
    entry.set_rw(true);
}

#[derive(Clone, Copy)]
#[repr(u64)]
pub enum AllocSize {
    S4KiB = (1 << 12),
    S2MiB = (1 << 21),
    S1GiB = (1 << 30),
}

impl AllocSize {
    pub fn size(&self) -> u64 {
        *self as u64
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

    pub unsafe fn alloc(&mut self, size: AllocSize, mut virt: u64, pages: usize, mut phys: u64) {
        match size {
            AllocSize::S4KiB => {
                for _ in 0..pages {
                    let (d0, d1, d2, d3, _) = Self::divide_virt_adr(virt);
                    self.alloc_map4((d0, d1, d2, d3), phys & !0xFFF);
                    virt += size.size();
                    phys += size.size();
                }
            }
            AllocSize::S2MiB => {
                for _ in 0..pages {
                    let (d0, d1, d2, _, _) = Self::divide_virt_adr(virt);
                    self.alloc_map3((d0, d1, d2), phys & !((1 << 21) - 1));
                    virt += size.size();
                    phys += size.size();
                }
            }
            AllocSize::S1GiB => {
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

const_assert!(size_of::<PageMapEntry>() == 8);
const_assert!(size_of::<PageMap>() == 4096);
assert_eq_size!(usize, u64);

pub unsafe fn new_kernel() -> *mut PageMap {
    let map = pmm::alloc_pages_zeroed(1) as *mut PageMap;
    let s4kib = 4 << 10;
    // Identity map the initial 4GiB of physical memory,
    (*map).alloc(AllocSize::S1GiB, 0, 4, 0);
    // Map heap memory.
    (*map).alloc(
        AllocSize::S4KiB,
        heap::VIRT_ADR as u64,
        heap::PAGE_COUNT,
        heap::PHYS_ADR as u64,
    );
    // Map video display memory
    (*map).alloc(
        AllocSize::S4KiB,
        fb::VIRT_ADR as u64,
        fb::PAGE_COUNT,
        fb::PHYS_ADR as u64,
    );
    // Map kernel to high address.
    let page_count = (30 << 20) / s4kib;
    let phys_adr = limine::kernel_address().physical_base;
    let virt_adr = 0xffffffff80000000 as u64;
    (*map).alloc(AllocSize::S4KiB, virt_adr, page_count, phys_adr);
    map
}

pub unsafe fn install(map: *mut PageMap) {
    PageMap::install_ptr(map)
}
