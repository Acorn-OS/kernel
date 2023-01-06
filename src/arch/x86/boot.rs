use core::arch::global_asm;

global_asm!(include_str!("boot/stage1.s"));
global_asm!(include_str!("boot/stage2.s"));
global_asm!(include_str!("boot/stage3.s"));
global_asm!(include_str!("boot/stage4.s"));
global_asm!(include_str!("boot/vga.s"));
global_asm!(include_str!("boot/gdt_defs.s"));

#[no_mangle]
unsafe extern "C" fn __rust_entry() -> ! {
    crate::kmain();
    unreachable!()
}
