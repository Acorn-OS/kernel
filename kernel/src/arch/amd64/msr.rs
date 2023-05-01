use core::arch::asm;

pub type Msr = u32;

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
