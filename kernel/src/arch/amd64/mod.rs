pub mod cpuc;
pub mod fb;
pub mod interrupt;
pub mod serial;
pub mod vm;

mod apic;
mod boot;
mod cpu;
mod gdt;
mod idt;
mod msr;
mod pic;
mod port;
mod sdt;

use crate::boot::BootInfo;
use apic::ioapic;

#[no_mangle]
pub unsafe fn arch_init(boot_info: &mut BootInfo) {
    trace!("initializing GDT");
    gdt::init();
    gdt::install();
    trace!("initializing IDT");
    idt::init();
    idt::install();
    trace!("initializing vm");
    vm::init();
    trace!("initializing core structures");
    cpuc::init_for_core();
    trace!("initializing the IO-APIC");
    let _ = sdt::init(&boot_info);
    ioapic::init();
    trace!("disabling PIC");
    pic::disable();
}
