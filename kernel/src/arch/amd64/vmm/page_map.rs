use core::arch::asm;
use core::mem::size_of;

bitfield! {
    #[derive(Clone, Copy)]
    #[repr(C, packed)]
    pub struct PageMapEntry(u64) {
        pub p :bool @ 0,
        pub rw: bool @ 1,
        pub us: bool @ 2,
        pub pwt: bool @ 3,
        pub pcd: bool @ 4,
        pub a: bool @ 5,
        pub ps: bool @ 7,
        _adr: u64 @ 12..=51,
        pub xd: bool @ 63,
    }
}

impl PageMapEntry {
    pub fn adr(&self) -> u64 {
        self._adr() << 12
    }

    pub fn set_adr(&mut self, adr: u64) {
        self.set__adr(adr >> 12)
    }
}

#[repr(C, align(4096))]
pub struct PageMap {
    entries: [PageMapEntry; 512],
}

pub trait AllocFn = FnMut() -> *mut PageMap;

mod get {
    use super::*;
    use crate::boot::BOOT_INFO;

    unsafe fn base(
        page_map: *mut PageMap,
        index: usize,
        alloc: &mut impl FnMut() -> *mut PageMap,
        adr_present: impl Fn(u64) -> u64,
        adr_new: impl Fn(u64) -> u64,
    ) -> *mut PageMap {
        debug_assert!(index < 512);
        let index = index & 0x1FF;
        if !(*page_map).entries[index].p() {
            let new = alloc();
            let page_entry = &mut (*page_map).entries[index];
            page_entry.set_adr(adr_new(new as u64));
            page_entry.set_p(true);
            page_entry.set_rw(true);
            new
        } else {
            adr_present((*page_map).entries[index].adr()) as *mut PageMap
        }
    }

    pub unsafe fn allocate(
        page_map: *mut PageMap,
        index: usize,
        alloc: &mut impl FnMut() -> *mut PageMap,
    ) -> *mut PageMap {
        base(page_map, index, alloc, |adr| adr, |adr| adr)
    }

    pub unsafe fn allocate_virt(
        page_map: *mut PageMap,
        index: usize,
        alloc: &mut impl FnMut() -> *mut PageMap,
    ) -> *mut PageMap {
        base(
            page_map,
            index,
            alloc,
            |adr| adr - BOOT_INFO.kernel_phys_base + BOOT_INFO.kernel_virt_base,
            |adr| adr - BOOT_INFO.kernel_virt_base + BOOT_INFO.kernel_phys_base,
        )
    }
}

unsafe fn set(page_map: *mut PageMap, index: usize, adr: u64) {
    debug_assert!(index < 512);
    let index = index & 0x1FF;
    let entry = &mut (*page_map).entries[index];
    entry.set_adr(adr);
    entry.set_p(true);
    entry.set_rw(true);
}

impl PageMap {
    pub const fn new() -> Self {
        Self {
            entries: [PageMapEntry(0); 512],
        }
    }

    pub unsafe fn install(&self) {
        Self::install_ptr(self as *const _)
    }

    pub unsafe fn install_ptr(ptr: *const Self) {
        unsafe {
            asm!(
                "mov cr3, rax",
                in("rax") ptr as u64,
                options(nostack)
            );
        }
    }

    pub unsafe fn alloc_map2(&mut self, index: (usize, usize), alloc: &mut impl AllocFn, adr: u64) {
        let ptr = self.as_mut_ptr();
        let (d0, d1) = index;
        set(get::allocate(ptr, d0, alloc), d1, adr);
    }

    pub unsafe fn alloc_map2_virt(
        &mut self,
        index: (usize, usize),
        alloc: &mut impl AllocFn,
        adr: u64,
    ) {
        let ptr = self.as_mut_ptr();
        let (d0, d1) = index;
        set(get::allocate_virt(ptr, d0, alloc), d1, adr);
    }

    pub unsafe fn alloc_map3(
        &mut self,
        index: (usize, usize, usize),
        alloc: &mut impl AllocFn,
        adr: u64,
    ) {
        let ptr = self.as_mut_ptr();
        let (d0, d1, d2) = index;
        set(
            get::allocate(get::allocate(ptr, d0, alloc), d1, alloc),
            d2,
            adr,
        );
    }

    pub unsafe fn alloc_map4(
        &mut self,
        index: (usize, usize, usize, usize),
        alloc: &mut impl AllocFn,
        adr: u64,
    ) {
        let ptr = self.as_mut_ptr();
        let (d0, d1, d3, d4) = index;
        set(
            get::allocate(
                get::allocate(get::allocate(ptr, d0, alloc), d1, alloc),
                d3,
                alloc,
            ),
            d4,
            adr,
        );
    }

    fn as_mut_ptr(&self) -> *mut PageMap {
        self as *const _ as *mut PageMap
    }
}

const_assert!(size_of::<PageMapEntry>() == 8);
const_assert!(size_of::<PageMap>() == 4096);
assert_eq_size!(usize, u64);
