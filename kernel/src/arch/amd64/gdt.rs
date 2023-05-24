use crate::mm::pmm;

use super::interrupt;
use core::arch::{asm, global_asm};
use core::fmt::Debug;
use core::mem::size_of;
use core::ptr::NonNull;

const KERNEL_CODE_ACCESS: u8 = 0x9a;
const KERNEL_CODE_FLAGS: u8 = 0xa;
const KERNEL_DATA_ACCESS: u8 = 0x92;
const KERNEL_DATA_FLAGS: u8 = 0xa;

const USRSPC_CODE_32_ACCESS: u8 = 0xfa;
const USRSPC_CODE_32_FLAGS: u8 = 0xc;
const USRSPC_CODE_ACCESS: u8 = 0xfa;
const USRSPC_CODE_FLAGS: u8 = 0xa;
const USRSPC_DATA_ACCESS: u8 = 0xf2;
const USRSPC_DATA_FLAGS: u8 = 0xa;

const TSS_ACCESS: u8 = 0x89;
const TSS_FLAGS: u8 = 0;

#[allow(unused)]
const ENTRY_SIZE: u16 = core::mem::size_of::<Entry>() as u16;
const_assert_eq!(ENTRY_SIZE, 8);

global_asm!(include_str!("gdt.s"));
extern "sysv64" {
    fn set_segments();
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct Entry(u64);

#[allow(dead_code)]
impl Entry {
    const fn new(base: u32, limit: u32, flags: u8, access: u8) -> Self {
        let base_lo = (base & 0xFFFF) as u64;
        let base_mid = ((base >> 16) & 0xFF) as u64;
        let base_hi = ((base >> 24) & 0xFF) as u64;
        let limit_lo = (limit & 0xFFFF) as u64;
        let limit_hi = ((limit >> 16) & 0xF) as u64;
        let flags = (flags & 0xF) as u64;
        let access = access as u64;
        let mut desc = 0;
        desc |= base_lo << 16;
        desc |= base_mid << 32;
        desc |= base_hi << 56;
        desc |= access << 40;
        desc |= limit_lo;
        desc |= limit_hi << 48;
        desc |= flags << 52;
        Self(desc)
    }

    const fn null() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct SegmentEntry(u128);

impl SegmentEntry {
    const fn null() -> Self {
        Self::new(0, 0, 0, 0)
    }

    const fn new(base: u64, limit: u32, flags: u8, access: u8) -> Self {
        let entry = Entry::new(base as u32, limit, flags, access);
        let base_ext = (base >> 32) as u128;
        Self(entry.0 as u128 | (base_ext << 64))
    }
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C, packed)]
pub struct Tss {
    pub _resv0: u32,
    pub rsp0: u64,
    pub rsp1: u64,
    pub rsp2: u64,
    pub _resv1: u64,
    pub ist1: u64,
    pub ist2: u64,
    pub ist3: u64,
    pub ist4: u64,
    pub ist5: u64,
    pub ist6: u64,
    pub ist7: u64,
    pub _resv2: u64,
    pub _resv3: u16,
    pub iopb: u16,
}

impl Tss {
    fn new() -> Self {
        Self::default()
    }
}

#[repr(C, packed)]
struct Gdtr {
    size: u16,
    adr: u64,
}

impl Debug for Gdtr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GDTR")
            .field("size", &format_args!("{}", { self.size }))
            .field("adr", &format_args!("0x{:016x}", { self.adr }))
            .finish()
    }
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct Gdt {
    // 0x00
    null: Entry,
    // 0x08
    kernel_code: Entry,
    // 0x10
    kernel_data: Entry,
    // 0x18
    usrspc_code_32: Entry,
    // 0x20
    usrspc_data: Entry,
    // 0x28
    usrspc_code: Entry,
    // 0x30
    tss: SegmentEntry,
}

pub const KERNEL_CODE_SELECTOR: u16 = offset_of!(Gdt, kernel_code) as u16;
pub const KERNEL_DATA_SELECTOR: u16 = offset_of!(Gdt, kernel_data) as u16;
pub const TSS_SELECTOR: u16 = offset_of!(Gdt, tss) as u16;
pub const USRSPC_CODE_32_SELECTOR: u16 = offset_of!(Gdt, usrspc_code_32) as u16;
pub const USRSPC_DATA_SELECTOR: u16 = offset_of!(Gdt, usrspc_data) as u16;
pub const USRSPC_CODE_SELECTOR: u16 = offset_of!(Gdt, usrspc_code) as u16;

impl Gdt {
    pub fn new(tss: NonNull<Tss>) -> Self {
        Self {
            null: Entry::null(),
            kernel_code: Entry::new(0, 0xffffff, KERNEL_CODE_FLAGS, KERNEL_CODE_ACCESS),
            kernel_data: Entry::new(0, 0xffffff, KERNEL_DATA_FLAGS, KERNEL_DATA_ACCESS),
            usrspc_code_32: Entry::new(0, 0xffffff, USRSPC_CODE_32_FLAGS, USRSPC_CODE_32_ACCESS),
            usrspc_data: Entry::new(0, 0xffffff, USRSPC_DATA_FLAGS, USRSPC_DATA_ACCESS),
            usrspc_code: Entry::new(0, 0xffffff, USRSPC_CODE_FLAGS, USRSPC_CODE_ACCESS),
            tss: SegmentEntry::new(
                tss.addr().get() as u64,
                size_of::<Tss>() as u32,
                TSS_FLAGS,
                TSS_ACCESS,
            ),
        }
    }

    pub unsafe fn install(&self) {
        interrupt::disable();
        let gdtr = self.to_gdtr();
        asm!(
            "lgdt [rax]",
            in("rax")(&gdtr),
            options(nostack)
        );
        set_segments();
    }

    pub unsafe fn use_tss(&self, descriptor: u16) {
        asm!(
            "ltr ax",
            in("ax") descriptor
        );
    }

    fn to_gdtr(&self) -> Gdtr {
        Gdtr {
            size: (core::mem::size_of::<Gdt>() - 1) as u16,
            adr: self as *const _ as u64,
        }
    }
}

static mut GDT: Gdt = Gdt {
    null: Entry::null(),
    kernel_code: Entry::null(),
    kernel_data: Entry::null(),
    tss: SegmentEntry::null(),
    usrspc_code_32: Entry::null(),
    usrspc_data: Entry::null(),
    usrspc_code: Entry::null(),
};

static mut TSS: Tss = Tss {
    _resv0: 0,
    rsp0: 0,
    rsp1: 0,
    rsp2: 0,
    _resv1: 0,
    ist1: 0,
    ist2: 0,
    ist3: 0,
    ist4: 0,
    ist5: 0,
    ist6: 0,
    ist7: 0,
    _resv2: 0,
    _resv3: 0,
    iopb: 0,
};

pub unsafe fn init() {
    trace!("initializing GDT");
    let rsp_stack_alloc = pmm::alloc_pages(16);
    let ist1_stack_alloc = pmm::alloc_pages(16);
    TSS = Tss {
        rsp0: rsp_stack_alloc.virt().adr(),
        rsp1: rsp_stack_alloc.virt().adr(),
        rsp2: rsp_stack_alloc.virt().adr(),
        ist1: ist1_stack_alloc.virt().adr(),
        ist2: 0,
        ist3: 0,
        ist4: 0,
        ist5: 0,
        ist6: 0,
        ist7: 0,
        iopb: 0,
        ..Tss::default()
    };
    GDT = Gdt::new(NonNull::new_unchecked(&mut TSS as *mut _));
}

pub unsafe fn install() {
    GDT.install();
    GDT.use_tss(TSS_SELECTOR);
}
