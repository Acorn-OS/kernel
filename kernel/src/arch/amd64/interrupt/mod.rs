pub mod idt;

mod except;
mod irq;
mod syscall;

use super::cpu::ctrl_regs::{cr0, cr4};
use super::gdt;
use super::vm::PageMapPtr;
use core::arch::{asm, global_asm};

global_asm!(include_str!("stubs.s"));

#[derive(Clone, Debug)]
#[repr(C, packed)]
pub struct StackFrame {
    cr4: u64,
    cr3: u64,
    cr0: u64,
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
    pub(super) fn zeroed() -> Self {
        Self {
            cr4: 0,
            cr3: 0,
            cr0: 0,
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
            rip: 0,
            cs: 0,
            rflags: 0,
            rsp: 0,
            ss: 0,
        }
    }

    pub fn new_kernel(ip: u64, sp: u64, page_map: PageMapPtr) -> Self {
        Self {
            cr4: cr4::get(),
            cr3: page_map.to_phys_adr().adr(),
            cr0: cr0::get(),
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

    pub fn new_userspace(ip: u64, sp: u64, page_map: PageMapPtr) -> Self {
        Self {
            cr4: cr4::get(),
            cr3: page_map.to_phys_adr().adr(),
            cr0: cr0::get(),
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
            cs: gdt::USRSPC_CODE_SELECTOR as u64 | 3,
            rflags: 1 << 9,
            rsp: sp,
            ss: gdt::USRSPC_DATA_SELECTOR as u64 | 3,
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

#[derive(Clone, Copy)]
struct IsrMeta {
    segment: u16,
    ist: u8,
    gate_type: idt::GateType,
}

static ISR_META_TBL: [IsrMeta; 256] = const {
    let mut tbl = [IsrMeta {
        segment: gdt::KERNEL_CODE_SELECTOR,
        ist: 0,
        gate_type: idt::GateType::Int,
    }; 256];
    // create exception metadata.
    let mut i = 0;
    while i < 32 {
        tbl[i] = IsrMeta {
            segment: gdt::KERNEL_CODE_SELECTOR,
            ist: 1,
            gate_type: idt::GateType::Int,
        };
        i += 1;
    }
    tbl
};

#[inline]
pub fn disable() {
    unsafe { asm!("cli", options(nostack)) };
}

#[inline]
pub fn enable() {
    unsafe { asm!("sti", options(nostack)) }
}

#[inline]
pub fn halt() {
    unsafe {
        asm!("hlt", options(nostack));
    }
}

#[inline]
pub fn is_enabled() -> bool {
    let rflags: u64;
    unsafe {
        asm!(
            "pushf",
            "pop rax",
            out("rax") rflags
        );
    }
    rflags & (1 << 9) != 0
}

pub unsafe fn init() {
    trace!("initializing interrupts");
    idt::init();
    idt::install();
    syscall::init();
}
