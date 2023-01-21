use core::arch::global_asm;

global_asm!(include_str!("x86/stage1.s"));
global_asm!(include_str!("x86/stage2.s"));
global_asm!(include_str!("x86/stage3.s"));
global_asm!(include_str!("x86/stage4.s"));
global_asm!(include_str!("x86/vga.s"));
global_asm!(include_str!("x86/gdt_defs.s"));

#[no_mangle]
unsafe extern "C" fn __rust_entry() -> ! {
    kernel::klog::init();
    kernel::mm::wm::init();
    kernel::mm::kalloc::init();
    unsafe {
        kernel::arch::init();
    }
    crate::main();
}
