use super::*;
use crate::mm::{heap, pmm};
use crate::{boot, logging};
use apic::ioapic;
use core::arch::global_asm;

global_asm!(include_str!("entry.s"));

#[no_mangle]
unsafe extern "C" fn amd64_boot() -> ! {
    interrupt::disable();
    logging::init();
    let mut boot_info = boot::info();
    pmm::init(&mut boot_info);
    trace!(
        "initialized physical memory management with '{}' pages",
        pmm::page_cnt()
    );
    trace!("initializing heap");
    heap::init();
    trace!("initializing core structures");
    cpuc::init_for_core();
    trace!("initializing the IO-APIC");
    let _ = sdt::init(&boot_info);
    ioapic::init();
    trace!("disabling PIC");
    pic::disable();
    trace!("entering kernel_early_main...");
    boot::kernel_early(&boot_info)
}
