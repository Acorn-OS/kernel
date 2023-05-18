use core::arch::asm;

pub type Msr = u32;

pub const IA32_EFER: Msr = 0xc0000080;
pub const IA32_STAR: Msr = 0xc0000081;
pub const IA32_LSTAR: Msr = 0xc0000082;
pub const IA32_FMASK: Msr = 0xc0000084;

pub const KERNEL_GS_BASE: u32 = 0xc0000102;
pub const GS_BASE: u32 = 0xc0000101;

pub const BASE_LAPIC_MSR: u32 = 0x1b;

pub fn get(msr: Msr) -> u64 {
    let lo: u32;
    let hi: u32;
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") msr as u32,
            out("eax") lo,
            out("edx") hi,
            options(nostack)
        )
    };
    lo as u64 | ((hi as u64) << 32)
}

pub fn set(msr: Msr, v: u64) {
    let lo = v as u32;
    let hi = (v >> 32) as u32;
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") msr as u32,
            in("eax") lo,
            in("edx") hi,
            options(nostack)
        )
    };
}
