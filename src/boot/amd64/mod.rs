use crate::{drivers, klog, ksyms, rust_main};
use amd64::{chipset, cpu};
use core::arch::global_asm;

global_asm!(include_str!("stage1.s"));
global_asm!(include_str!("stage2.s"));
global_asm!(include_str!("stage3.s"));
global_asm!(include_str!("stage4.s"));
global_asm!(include_str!("vga.s"));
global_asm!(include_str!("gdt_defs.s"));
global_asm!(include_str!("paging.s"));
global_asm!(include_str!("disk.s"));

const IRQ_START_VEC: u8 = 0x20;
const EXCEPT_START_VEC: u8 = 0x00;

#[no_mangle]
unsafe extern "C" fn __rust_entry() -> ! {
    util::di();

    // Initializes com drivers for serial printing.
    drivers::com::init();

    klog::init();

    cpu::isr::init_excepts(EXCEPT_START_VEC);
    cpu::isr::init_irqs(IRQ_START_VEC);
    chipset::pic::remap(IRQ_START_VEC, IRQ_START_VEC + 8);
    chipset::pic::enable_all();
    cpu::idt::install();

    cpu::paging::init(ksyms::root_pt());

    // Remap kernel into virtual memory.
    cpu::paging::map(
        ksyms::virt_adr_start()..=ksyms::virt_adr_end(),
        0,
        cpu::paging::PageSize::Huge,
    );
    // Identity map kernel memory.
    cpu::paging::map(
        0..=ksyms::free_mem_adr() + ksyms::free_mem_len() - 1,
        0,
        cpu::paging::PageSize::Large,
    );
    unsafe { cpu::paging::install() };

    unwinding::panic::catch_unwind(|| {
        rust_main();
    })
    .expect("unwinding.");
    loop {}
}
