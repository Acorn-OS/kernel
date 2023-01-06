use crate::{
    arch::{self},
    math, mm,
};
use core::{arch::asm, mem::size_of};

const P1_SHFT: usize = 12;
const P2_SHFT: usize = 21;
const P3_SHFT: usize = 30;
const P4_SHFT: usize = 39;

const STEPS: usize = 1 << P1_SHFT;
const STEPS_HUGE: usize = 1 << P3_SHFT;

bitfield! {
    #[derive(Clone, Copy)]
    struct Entry(u64): FromRaw {
        pub p: bool @ 0,
        pub rw: bool @ 1,
        pub us: bool @ 2,
        pub pwt: bool @ 3,
        pub pcd: bool @ 4,
        pub a: bool @ 5,
        pub d: bool @ 6,
        pub ps: bool @ 7,
        pub g: bool @ 8,
        pub avl: u64 @ 9..=11,
        pub pat: bool @ 12,
        adr_internal: u64 @ 12..=58,
        pub pk: u64 @ 59..=62,
        pub xd: bool @ 63
    }
}

const_assert!(size_of::<Entry>() == 8);
const_assert!(size_of::<PageDirectory>() == 4096);
assert_eq_size!(usize, u64);

impl Entry {
    #[inline(always)]
    fn adr(&self) -> u64 {
        ((((self.adr_internal() << 12) as i64) << 11) >> 11) as u64
    }

    #[inline(always)]
    fn set_adr(&mut self, adr: u64) {
        self.set_adr_internal(adr >> 12)
    }

    #[inline]
    fn enable_rwp(&mut self) {
        self.set_rw(true);
        self.set_p(true);
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.0 & 0b1 == 0
    }
}

#[repr(C, align(4096))]
struct PageDirectory {
    entries: [Entry; 512],
}

impl PageDirectory {
    fn from_ptr(map: PageMapPtr) -> *mut Self {
        map as *mut _
    }
}

fn get_next_with_alloc(map: *mut PageDirectory, index: usize) -> *mut PageDirectory {
    debug_assert!(index < 512);
    let entry = unsafe { (*map).entries.get_unchecked_mut(index & 0x1FF) };
    if entry.is_empty() {
        entry.enable_rwp();
        unsafe {
            entry.set_adr(alloc::alloc::alloc(alloc::alloc::Layout::new::<PageDirectory>()) as u64)
        };
        entry.adr() as *mut _
    } else {
        entry.adr() as *mut _
    }
}

fn get_next_with_free(map: *mut PageDirectory, index: usize) {
    unimplemented!()
}

fn deconstruct_virt(virt: usize) -> (usize, usize, usize, usize) {
    let p4 = (virt & (0x1FF << P4_SHFT)) >> P4_SHFT;
    let p3 = (virt & (0x1FF << P3_SHFT)) >> P3_SHFT;
    let p2 = (virt & (0x1FF << P2_SHFT)) >> P2_SHFT;
    let p1 = (virt & (0x1FF << P1_SHFT)) >> P1_SHFT;
    (p4, p3, p2, p1)
}

#[inline]
fn vmap(base: *mut PageDirectory, virt: usize, phys: usize, flags: MapFlags) {
    let (p4i, p3i, p2i, p1i) = deconstruct_virt(virt);
    let p4 = base;
    let entry = if flags.huge {
        let p3 = get_next_with_alloc(p4, p4i);
        let entry = unsafe { &mut (*p3).entries[p3i & 0x1FF] };
        entry.set_ps(true);
        entry
    } else {
        let p3 = get_next_with_alloc(p4, p4i);
        let p2 = get_next_with_alloc(p3, p3i);
        let p1 = get_next_with_alloc(p2, p2i);
        unsafe { &mut (*p1).entries[p1i & 0x1FF] }
    };
    entry.enable_rwp();
    entry.set_adr(phys as u64);
}

pub fn new_map() -> PageMapPtr {
    mm::vm::kv2p(unsafe {
        alloc::alloc::alloc(alloc::alloc::Layout::new::<PageDirectory>()) as PageMapPtr
    })
}

pub fn destroy_map(map: PageMapPtr) {
    unimplemented!()
}

pub fn swap_map(map: PageMapPtr) {
    unsafe {
        asm!(
            "mov cr3, rax",
            in("rax") map as u64,
            options(nostack)
        );
    }
}

pub fn map(map: PageMapPtr, virt: usize, phys: usize, flags: arch::mm::MapFlags) -> Result<(), ()> {
    vmap(PageDirectory::from_ptr(map), virt, phys, flags);
    Ok(())
}

pub fn map_range(
    map: PageMapPtr,
    virt: usize,
    len: usize,
    phys: usize,
    flags: MapFlags,
) -> Result<(), ()> {
    let mut mapped_len = 0;
    let base = PageDirectory::from_ptr(map);
    let mut nvirt = virt;
    let mut nphys = phys;
    let steps = if flags.huge { STEPS_HUGE } else { STEPS };
    let len = math::align_ceil(len, steps);
    while mapped_len < len && nvirt >= virt && nphys >= phys {
        vmap(base, nvirt, nphys, flags);
        nphys = nphys.wrapping_add(steps);
        nvirt = nvirt.wrapping_add(steps);
        mapped_len += steps;
    }
    Ok(())
}

pub fn unmap(vadr: usize) -> usize {
    unimplemented!()
}

pub fn mapped() -> bool {
    unimplemented!()
}

pub fn gmmap() {}
