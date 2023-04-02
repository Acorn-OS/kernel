use super::*;
use crate::boot::kernel_early;
use crate::mm::{heap, pmm};
use crate::{logging, util};
use core::arch::global_asm;
use gdt::Gdt;
use idt::Idt;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
unsafe extern "C" fn amd64_boot() -> ! {
    util::irq_di();
    logging::init();
    trace!("initializing physical memory management");
    pmm::init();
    trace!("initializing virtual memory");
    let vmm_map = vm::new_kernel().expect("unable to create a page table for the main core");
    vm::install(vmm_map);
    trace!("initializing the kernel heap");
    heap::init();
    trace!("initializing GDT");
    let gdt = heap::alloc(Gdt::new());
    (*gdt).install();
    trace!("initializing IDT");
    let idt = heap::alloc(Idt::new());
    (*idt).install();
    //trace!("parsing SDTs");
    //assert!(sdt::validate(), "the RSDT could not be validated");
    //trace!("disabling PIC");
    //pic::disable();
    //trace!("initializing LAPIC");
    //lapic::init_locally(&mut *vmm_map);
    //trace!("initializing the IO-APIC");
    //ioapic::init();
    kernel_early()
}
