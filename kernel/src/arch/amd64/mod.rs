pub mod cpu;
pub mod fb;
pub mod interrupt;
pub mod panic;
pub mod serial;
pub mod stack_unwind;
pub mod thread;
pub mod vm;

mod apic;
mod gdt;
mod msr;
mod pic;
mod port;
mod sdt;

use crate::boot::BootInfo;
use apic::ioapic;
use core::arch::global_asm;

global_asm!(include_str!("boot.s"));

#[allow(non_camel_case_types)]
pub type vadr = u64;
#[allow(non_camel_case_types)]
pub type padr = u64;

#[no_mangle]
pub unsafe fn arch_init(boot_info: &mut BootInfo) {
    gdt::init();
    gdt::install();
    interrupt::init();
    vm::init();
    cpu::init_core();
    let _ = sdt::init(&boot_info);
    ioapic::init();
    pic::disable();
}
