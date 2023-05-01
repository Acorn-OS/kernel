use super::interrupt;
use core::arch::{asm, global_asm};
use core::fmt::Debug;

const KERNEL_CODE_ACCESS: u8 = 0x9a;
const KERNEL_CODE_FLAGS: u8 = 0xa;
const KERNEL_DATA_ACCESS: u8 = 0x92;
const KERNEL_DATA_FLAGS: u8 = 0xa;

const USRSPC_CODE_ACCESS: u8 = 0xfa;
const USRSPC_CODE_FLAGS: u8 = 0xa;
const USRSPC_DATA_ACCESS: u8 = 0xf2;
const USRSPC_DATA_FLAGS: u8 = 0xa;

#[allow(unused)]
const ENTRY_SIZE: u16 = core::mem::size_of::<Entry>() as u16;
const_assert_eq!(ENTRY_SIZE, 8);

pub const KERNEL_CODE_SELECTOR: u16 = ENTRY_SIZE;
pub const KERNEL_DATA_SELECTOR: u16 = ENTRY_SIZE * 2;
pub const USRSPC_CODE_SELECTOR: u16 = ENTRY_SIZE * 3;
pub const USRSPC_DATA_SELECTOR: u16 = ENTRY_SIZE * 4;

global_asm!(include_str!("gdt.s"));
extern "sysv64" {
    fn set_segments();
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct Entry(u64);

#[allow(dead_code)]
impl Entry {
    const fn new(base: u32, limit: u32, flags: u8, access: u8) -> Self {
        let base_lo = (base & 0xFFFF) as u64;
        let base_mid = ((base >> 16) & 0xF) as u64;
        let base_hi = ((base >> 20) & 0xF) as u64;
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

impl Debug for Entry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("0x{:016x}", { self.0 }))
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
    usrspc_code: Entry,
    // 0x20
    usrspc_data: Entry,
}

impl Gdt {
    pub fn new() -> Self {
        Self {
            null: Entry::null(),
            kernel_code: Entry::new(0, 0xffffff, KERNEL_CODE_FLAGS, KERNEL_CODE_ACCESS),
            kernel_data: Entry::new(0, 0xffffff, KERNEL_DATA_FLAGS, KERNEL_DATA_ACCESS),
            usrspc_code: Entry::new(0, 0xffffff, USRSPC_CODE_FLAGS, USRSPC_CODE_ACCESS),
            usrspc_data: Entry::new(0, 0xffffff, USRSPC_DATA_FLAGS, USRSPC_DATA_ACCESS),
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

    fn to_gdtr(&self) -> Gdtr {
        Gdtr {
            size: (core::mem::size_of::<Gdt>() - 1) as u16,
            adr: self as *const _ as u64,
        }
    }
}
