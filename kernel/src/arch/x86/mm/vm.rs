use crate::arch::mm::{pptr, vptr};
use core::arch::asm;
use core::mem::size_of;

const KPG_L: usize = 30;
/// Large page size for kernel.
const KPGS_L: usize = 1 << KPG_L;
const KPG_S: usize = 12;
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
        pub xd: bool @ 63
    }
}

const_assert!(size_of::<Entry>() == 8);
const_assert!(size_of::<PageDirectory>() == 4096);
assert_eq_size!(usize, u64);

impl Entry {
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
        unsafe {
            let virt = alloc::alloc::alloc(alloc::alloc::Layout::new::<PageDirectory>()) as usize;
            let phys = get_phys(virt).expect("unmapped allocation.");
            entry.set_adr(phys as u64);
        };
        entry.adr() as *mut _
    }
}

fn get_next_with_free(map: *mut PageDirectory, index: usize) {
    unimplemented!()
}

fn get_phys(virt: vptr) -> Option<pptr> {
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

static mut KERNEL_PT4: PageDirectory = PageDirectory([Entry(0); 512]);

fn kt_ptr() -> *mut PageDirectory {
    unsafe { (&mut KERNEL_PT4) as *mut PageDirectory }
}

pub fn map(virt: vptr, phys: pptr) {
    vmap(kt_ptr(), virt, phys, false)
}

pub fn unmap(virt: usize) {
    unimplemented!()
}

use super::adr::{KVIRT_BEG, KVIRT_END};
pub fn init() {
    let mut virt: u128 = KVIRT_BEG as u128;
    let mut phys = 0;
    while virt < KVIRT_END as u128 {
        let adr = virt as vptr;
        // Map kernel to high memory.
        vmap(kt_ptr(), adr, phys, true);
        // Identity map kernel.
        vmap(kt_ptr(), phys as usize, phys, true);
        phys += KPGS_L as u64;
        virt += KPGS_L as u128;
    }
    let map = kt_ptr() as usize - KVIRT_BEG;
    unsafe {
        asm!(
            "mov cr3, rax",
            in("rax") map as u64,
            options(nostack)
        );
    }
}
