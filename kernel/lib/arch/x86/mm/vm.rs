use crate::arch::mm::{pptr, vptr};
use crate::mm::wm;
use core::arch::asm;
use core::mem::size_of;

const KPG_L: usize = P3_SHFT;
/// Large page size for kernel.
const KPGS_L: usize = 1 << KPG_L;
const KPG_S: usize = P1_SHFT;
/// Small page size for kernel.
const KPGS_S: usize = 1 << KPG_S;

const P1_SHFT: usize = 12;
const P2_SHFT: usize = 21;
const P3_SHFT: usize = 30;
const P4_SHFT: usize = 39;

bitfield! {
    #[derive(Clone, Copy)]
    struct Entry(u64): Debug, FromRaw {
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
        adr_internal: u64 @ 12..=51,
        pub pk: u64 @ 59..=62,
        pub xd: bool @ 63,
    }
}

const_assert!(size_of::<Entry>() == 8);
const_assert!(size_of::<PageDirectory>() == 4096);
assert_eq_size!(usize, u64);

impl Entry {
    const fn new_empty() -> Self {
        Self(0)
    }

    #[inline(always)]
    fn adr(&self) -> u64 {
        ((((self.adr_internal() << 12) as i64) << 12) >> 12) as u64
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
}

#[repr(C, align(4096))]
struct PageDirectory([Entry; 512]);

#[repr(align(4096))]
struct PageTableBank<const TABLES: usize> {
    base: PageDirectory,
    tables_occupied: [bool; TABLES],
    tables: [PageDirectory; TABLES],
}

const TABLES: usize = 8192;
type Bank = PageTableBank<TABLES>;
static mut TBL_BANK: usize = 0;

fn bank<T>(mut f: impl FnMut(&'static mut Bank) -> T) -> T {
    util::guard! {
        let r = unsafe { &mut *(TBL_BANK as *mut _) };
        f(r)
    }
}

impl<const TABLES: usize> PageTableBank<TABLES> {
    fn resv_first_empty(&mut self) -> Option<*mut PageDirectory> {
        for i in 0..TABLES {
            let index = i % TABLES;
            if !self.tables_occupied[index] {
                self.tables_occupied[index] = true;
                return Some(self.get_ptr_from_index(index));
            }
        }
        None
    }

    fn get_ptr_from_index(&mut self, index: usize) -> *mut PageDirectory {
        debug_assert!(index < TABLES);
        &mut self.tables[index % TABLES] as *mut _
    }

    fn free(&mut self, adr: usize) {
        unimplemented!()
    }
}

fn bank_base() -> *mut PageDirectory {
    bank(|bank| &mut bank.base as *mut _)
}

fn bank_next() -> *mut PageDirectory {
    bank(|bank| {
        bank.resv_first_empty()
            .expect("ran out of space in page bank")
    })
}

fn bank_free(adr: usize) {
    bank(|bank| bank.free(adr))
}

fn get_cur_base_map() -> *mut PageDirectory {
    let adr: u64;
    unsafe {
        asm!(
            "mov rax, cr3",
            out("rax") adr,
            options(nostack)
        );
    }
    adr as *mut _
}

fn deconstruct_virt(virt: vptr) -> (usize, usize, usize, usize) {
    let p4 = (virt & (0x1FF << P4_SHFT)) >> P4_SHFT;
    let p3 = (virt & (0x1FF << P3_SHFT)) >> P3_SHFT;
    let p2 = (virt & (0x1FF << P2_SHFT)) >> P2_SHFT;
    let p1 = (virt & (0x1FF << P1_SHFT)) >> P1_SHFT;
    (p4, p3, p2, p1)
}

fn get_next_with_alloc(map: *mut PageDirectory, index: usize) -> *mut PageDirectory {
    debug_assert!(index < 512);
    let entry = unsafe { &mut (*map).0[index & 0x1FF] };
    if entry.p() {
        entry.adr() as *mut _
    } else {
        entry.enable_rwp();
        let virt = bank_next();
        let phys = vtop(virt as usize).expect("unable to map virtual address to physical address"); // works because of identity mapping;
        entry.set_adr(phys as u64);
        entry.adr() as *mut _
    }
}

fn get_next_with_free(map: *mut PageDirectory, index: usize) {
    unimplemented!()
}

/// virtual to physical.
fn vtop(virt: vptr) -> Option<pptr> {
    let offset1 = virt as pptr & ((1 << P1_SHFT) - 1);
    let offset2 = virt as pptr & ((1 << P2_SHFT) - 1);
    let offset3 = virt as pptr & ((1 << P3_SHFT) - 1);
    let offset4 = virt as pptr & ((1 << P4_SHFT) - 1);
    let (p4i, p3i, p2i, p1i) = deconstruct_virt(virt);
    macro_rules! get {
        ($map:expr, $index:expr, $offset:expr) => {{
            debug_assert!($index < 512);
            let entry = unsafe { (*$map).0[$index & 0x1FF] };
            if entry.p() {
                let ptr = entry.adr() as *mut PageDirectory;
                if entry.ps() {
                    return Some(ptr as pptr + $offset);
                } else {
                    Some(ptr)
                }
            } else {
                None
            }
        }?};
    }
    let p4 = get_cur_base_map();
    let p3 = get!(p4, p4i, offset4);
    let p2 = get!(p3, p3i, offset3);
    let p1 = get!(p2, p2i, offset2);
    let entry = unsafe { (*p1).0[p1i & 0x1FF] };
    if entry.p() {
        Some(entry.adr() + offset1)
    } else {
        None
    }
}

#[inline]
fn vmap(base: *mut PageDirectory, virt: vptr, phys: pptr, large_pg_size: bool) {
    let (p4i, p3i, p2i, p1i) = deconstruct_virt(virt);
    let p4 = base;
    let entry = if large_pg_size {
        let p3 = get_next_with_alloc(p4, p4i);
        let entry = unsafe { &mut (*p3).0[p3i & 0x1FF] };
        entry.set_ps(true);
        entry
    } else {
        let p3 = get_next_with_alloc(p4, p4i);
        let p2 = get_next_with_alloc(p3, p3i);
        let p1 = get_next_with_alloc(p2, p2i);
        unsafe { &mut (*p1).0[p1i & 0x1FF] }
    };
    entry.enable_rwp();
    entry.set_adr(phys as u64);
}

pub fn map(virt: vptr, phys: pptr) {
    vmap(bank_base(), virt, phys, false)
}

pub fn unmap(virt: usize) {
    unimplemented!()
}

use super::adr::{KVIRT_BEG, KVIRT_END};
/// # Safety
/// Access mutable static.
pub unsafe fn init() {
    TBL_BANK = wm::reserve::<Bank>(|| Bank {
        base: PageDirectory([Entry::new_empty(); _]),
        tables_occupied: [false; _],
        tables: ::core::mem::MaybeUninit::zeroed().assume_init(),
    }) as usize;
    let mut virt = KVIRT_BEG;
    let mut phys = 0;
    let virt_initial = virt;
    while virt < KVIRT_END && virt >= virt_initial {
        // Map kernel to high memory.
        vmap(bank_base(), virt as vptr, phys, true);
        // Identity map kernel.
        vmap(bank_base(), phys as vptr, phys, true);
        phys += KPGS_L as u64;
        virt = virt.wrapping_add(KPGS_L);
    }
    let map = vtop(bank_base() as usize).expect("main page table isn't mapped");
    unsafe {
        asm!(
            "mov cr3, rax",
            in("rax") map as u64,
            options(nostack)
        );
    }
}
