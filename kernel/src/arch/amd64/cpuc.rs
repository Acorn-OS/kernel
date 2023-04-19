use crate::mm::vmm::VirtualMemory;

use super::gdt::GDT;
use super::idt::IDT;
use super::lapic::LAPICPtr;
use super::msr;
use core::arch::asm;

const KERNEL_GS_BASE: u32 = 0xC0000102;
const GS_BASE: u32 = 0xC0000101;

#[repr(C)]
pub struct Core {
    pub(super) lapic_ptr: LAPICPtr,
    pub(super) idt_ptr: *mut IDT,
    pub(super) gdt_ptr: *mut GDT,
    pub(super) vmm_ptr: *mut VirtualMemory,
}

impl Core {
    pub fn vmm(&self) -> *mut VirtualMemory {
        self.vmm_ptr
    }

    pub fn set_vmm(&mut self, ptr: *mut VirtualMemory) {
        self.vmm_ptr = ptr;
    }
}

pub unsafe fn swap() {
    asm!("swapgs");
}

pub unsafe fn set_kernel_gs_base(ptr: *mut Core) {
    msr::set(KERNEL_GS_BASE, ptr as u64);
}

pub unsafe fn set_gs_base(ptr: *mut Core) {
    msr::set(GS_BASE, ptr as u64);
}

pub fn get() -> *mut Core {
    msr::get(KERNEL_GS_BASE) as *mut _
}
