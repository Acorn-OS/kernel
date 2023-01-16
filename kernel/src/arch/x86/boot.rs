use core::arch::global_asm;

global_asm!(include_str!("boot/stage1.s"));
global_asm!(include_str!("boot/stage2.s"));
global_asm!(include_str!("boot/stage3.s"));
global_asm!(include_str!("boot/stage4.s"));
global_asm!(include_str!("boot/vga.s"));
global_asm!(include_str!("boot/gdt_defs.s"));

unsafe fn pre_init() {
    crate::klog::init();
    super::serial::init();
    trace!("pre init.");
    trace!("mm init");
    crate::mm::init();
    trace!("vm init.");
    super::mm::vm::init();
}

unsafe fn init() {
    trace!("init");
    trace!("init segmentation.");
    super::segments::init();
    trace!("init irq/exception.");
    super::irq::init();
    trace!("init framebuffer.");
    super::fb::init();
}

unsafe fn post_init() {
    trace!("post init");
}

#[no_mangle]
unsafe extern "C" fn __rust_entry() -> ! {
    unsafe {
        pre_init();
        init();
        post_init();
    }
    log::trace!("entering kmain...");
    crate::kmain()
}
