use super::*;
use crate::boot;
use crate::mm::{heap, pmm, vmm};
use crate::{logging, util};
use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
unsafe extern "C" fn amd64_boot() -> ! {
    util::irq_di();
    logging::init();
    let mmap_mut = boot::get_mmap();
    trace!("initializing physical memory management");
    pmm::init(mmap_mut);
    trace!("creating new kernel vmm object");
    let mut vmm = vmm::new_kernel();
    trace!("installing vmm object");
    vmm.install();
    trace!("initializing GDT");
    let gdt = gdt::new(&mut vmm);
    (*gdt).install();
    trace!("initializing IDT");
    let idt = idt::new(&mut vmm);
    (*idt).install();
    trace!("initializing heap");
    heap::init();
    loop {}
    trace!("parsing SDTs");
    assert!(sdt::validate(), "the RSDT could not be validated");
    trace!("disabling PIC");
    pic::disable();
    trace!("initializing LAPIC");
    lapic::create_local(&mut vmm);
    trace!("initializing the IO-APIC");
    ioapic::init();
    boot::kernel_early()
}
