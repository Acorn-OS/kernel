use core::arch::{asm, global_asm};

#[repr(C, packed)]
struct Entry(u64);

#[allow(dead_code)]
impl Entry {
    const fn new(base: u32, limit: u32, flags: u8, access: u8) -> Self {
        let mut desc = 0;
        let base_lo = (base & 0xFFFF) as u64;
        let base_mid = ((base >> 16) & 0xF) as u64;
        let base_hi = ((base >> 20) & 0xF) as u64;
        let limit_hi = ((limit >> 16) & 0xF) as u64;
        let limit_lo = (limit & 0xFFFF) as u64;
        let flags = (flags & 0xF) as u64;
        let access = access as u64;
        desc |= base_mid;
        desc |= access << 8;
        desc |= limit_hi << 16;
        desc |= flags << 20;
        desc |= base_hi << 24;
        desc <<= 32;
        desc |= limit_lo;
        desc |= base_lo << 16;
        Self(desc)
    }
}

const ENTRY_SIZE: u16 = core::mem::size_of::<Entry>() as u16;
const_assert_eq!(ENTRY_SIZE, 8);

#[allow(unused)]
pub(crate) const KERNEL_CODE_SELECTOR: u16 = ENTRY_SIZE * 1;
#[allow(unused)]
pub(crate) const KERNEL_DATA_SELECTOR: u16 = ENTRY_SIZE * 2;
#[allow(unused)]
pub(crate) const USRSPC_CODE_SELECTOR: u16 = ENTRY_SIZE * 3;
#[allow(unused)]
pub(crate) const USRSPC_DATA_SELECTOR: u16 = ENTRY_SIZE * 4;

const KERNEL_CODE_ACCESS: u8 = 0x9A;
const KERNEL_CODE_FLAGS: u8 = 0xA;
const KERNEL_DATA_ACCESS: u8 = 0x92;
const KERNEL_DATA_FLAGS: u8 = 0xC;

const USER_CODE_ACCESS: u8 = 0xFA;
const USER_CODE_FLAGS: u8 = 0xA;
const USER_DATA_ACCESS: u8 = 0xF2;
const USER_DATA_FLAGS: u8 = 0xC;

#[repr(C, packed)]
struct GDTR {
    size: u16,
    adr: u64,
}

#[allow(dead_code)]
struct GDT {
    entries: [Entry; 5],
}

#[allow(dead_code)]
impl GDT {
    const fn new() -> Self {
        Self {
            entries: [
                // null entry
                Entry::new(0, 0, 0, 0),
                // kernel code
                Entry::new(0, 0, KERNEL_CODE_FLAGS, KERNEL_CODE_ACCESS),
                // kernel data
                Entry::new(0, 0, KERNEL_DATA_FLAGS, KERNEL_DATA_ACCESS),
                // userspace code
                Entry::new(0, 0, USER_CODE_FLAGS, USER_CODE_ACCESS),
                // userspace data
                Entry::new(0, 0, USER_DATA_FLAGS, USER_DATA_ACCESS),
            ],
        }
    }

    fn to_gdtr(&self) -> GDTR {
        GDTR {
            size: (core::mem::size_of::<GDT>() - 1) as u16,
            adr: &self as *const _ as u64,
        }
    }
}

global_asm!(include_str!("segments.s"));
extern "C" {
    fn set_segments(cs: u16, ds: u16, ss: u16, es: u16, gs: u16, fs: u16);
}

static mut GDT: GDT = GDT::new();

pub unsafe fn init() {
    set_segments(
        KERNEL_CODE_SELECTOR,
        KERNEL_DATA_SELECTOR,
        KERNEL_DATA_SELECTOR,
        KERNEL_DATA_SELECTOR,
        KERNEL_DATA_SELECTOR,
        KERNEL_DATA_SELECTOR,
    );

    let gdtr = GDT.to_gdtr();
    asm!(
        "lgdt [rax]",
        in("rax")(&gdtr)
    );
}
