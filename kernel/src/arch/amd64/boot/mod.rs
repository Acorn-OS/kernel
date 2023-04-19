use super::*;
use crate::arch::imp::gdt::GDT;
use crate::arch::imp::idt::IDT;
use crate::boot;
use crate::mm::{heap, hhdm, pmm, vmm};
use crate::{logging, util};
use core::arch::global_asm;
use core::ptr::null_mut;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
unsafe extern "C" fn amd64_boot() -> ! {
    util::irq_di();
    logging::init();
    let mut boot_info = boot::info();
    trace!("initializing physical memory management");
    pmm::init(&mut boot_info);
    hhdm::init(&boot_info);
    trace!("initializing heap");
    heap::init();
    trace!("creating new kernel vmm object");
    let vmm_ptr = heap::alloc(vmm::new_kernel());
    trace!("installing vmm object");
    (*vmm_ptr).install();
    trace!("initializing GDT");
    let gdt_ptr = heap::alloc(GDT::new());
    (*gdt_ptr).install();
    trace!("initializing IDT");
    let idt_ptr = heap::alloc(IDT::new());
    (*idt_ptr).install();
    trace!("initializing LAPIC");
    let lapic_ptr = lapic::create_local(&mut *vmm_ptr);
    trace!("setting up cpuc object");
    cpuc::set_kernel_gs_base(heap::alloc(cpuc::Core {
        lapic_ptr,
        idt_ptr,
        gdt_ptr,
        vmm_ptr,
    }));
    cpuc::set_gs_base(null_mut());
    trace!("initializing the IO-APIC");
    let rsdp = rsdp::get(&boot_info);
    let rsdt = sdt::get_base(&rsdp);
    assert!(sdt::validate(rsdt), "the RSDT could not be validated");
    ioapic::init();
    trace!("disabling PIC");
    pic::disable();
    trace!("booting kernel main");
    boot::kernel_early()
}
