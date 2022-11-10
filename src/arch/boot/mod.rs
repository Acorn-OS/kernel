use crate::{arch, rust_main, util};
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
    arch::isr::init_excepts(EXCEPT_START_VEC);
    arch::isr::init_irqs(IRQ_START_VEC);
    arch::pic::remap(IRQ_START_VEC, IRQ_START_VEC + 8);
    arch::pic::enable_all();
    arch::idt::install();
    rust_main()
}
