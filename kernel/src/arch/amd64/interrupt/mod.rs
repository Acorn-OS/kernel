mod except;
mod irq;

use super::gdt;
use core::arch::global_asm;

global_asm!(include_str!("stubs.s"));

#[repr(C, packed)]
pub struct StackFrame {
    rbp: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64,
    error: u64,
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

impl StackFrame {
    pub fn new_kernel(ip: u64, sp: u64) -> Self {
        Self {
            rbp: 0,
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            r11: 0,
            r10: 0,
            r9: 0,
            r8: 0,
            rdi: 0,
            rsi: 0,
            rdx: 0,
            rcx: 0,
            rbx: 0,
            rax: 0,
            error: 0,
            rip: ip,
            cs: gdt::KERNEL_CODE_SELECTOR as u64,
            rflags: 1 << 9,
            rsp: sp,
            ss: gdt::KERNEL_DATA_SELECTOR as u64,
        }
    }

    pub fn new_userspace(ip: u64, sp: u64) -> Self {
        Self {
            rbp: 0,
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            r11: 0,
            r10: 0,
            r9: 0,
            r8: 0,
            rdi: 0,
            rsi: 0,
            rdx: 0,
            rcx: 0,
            rbx: 0,
            rax: 0,
            error: 0,
            rip: ip,
            cs: gdt::USRSPC_CODE_SELECTOR as u64,
            rflags: 1 << 9,
            rsp: sp,
            ss: gdt::USRSPC_DATA_SELECTOR as u64,
        }
    }
}

#[no_mangle]
unsafe extern "C" fn unimp() -> ! {
    panic!("unimplemented handler");
}

extern "C" {
    pub static irq_routines: [extern "C" fn(); 256];
}

pub fn disable() {
    unsafe { core::arch::asm!("cli") };
}

pub fn enable() {
    unsafe { core::arch::asm!("sti") }
}

pub fn halt() {
    unsafe {
        core::arch::asm!("hlt");
    }
}