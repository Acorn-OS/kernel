use core::arch::global_asm;
use mm::vm;
use util::info;

global_asm!(include_str!("stage1.s"));
global_asm!(include_str!("stage2.s"));
global_asm!(include_str!("stage3.s"));
global_asm!(include_str!("stage4.s"));
global_asm!(include_str!("vga.s"));
global_asm!(include_str!("gdt_defs.s"));
global_asm!(include_str!("paging.s"));
global_asm!(include_str!("disk.s"));

fn serial_log(str: &str) {
    amd64::serial::puts(str);
}

fn fb_log(str: &str) {
    unsafe {
        amd64::framebuffer::puts(str, amd64::framebuffer::Color::WHITE);
    }
}

#[no_mangle]
unsafe extern "C" fn __rust_entry() -> ! {
    util::di();

    amd64::serial::init();

    kernel::klog::configure(serial_log, fb_log);
    kernel::klog::init();

    mm::alloc::init();

    amd64::segments::init();
    amd64::irq::init();
    amd64::framebuffer::clear();

    info!(
        "mapping kernel into virtual address [0x{:016X}, 0x{:016X}]",
        vm::kvma_start(),
        vm::kvma_end()
    );

    // Remap kernel into virtual memory.
    amd64::mm::paging::map(
        vm::kvma_start()..=vm::kvma_end(),
        0,
        amd64::mm::paging::PageSize::Huge,
    );
    // Identity map kernel memory.
    amd64::mm::paging::map(0..=vm::kwm_end(), 0, amd64::mm::paging::PageSize::Large);
    unsafe { amd64::mm::paging::install() };

    kernel::kmain();
}
