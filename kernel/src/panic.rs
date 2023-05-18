use crate::arch::interrupt;
use crate::arch::stack_unwind::StackFrame;
use crate::kernel_elf;
use core::panic::PanicInfo;

#[inline(always)]
unsafe fn unwind_stack() {
    let mut stack_frame = StackFrame::from_current_stackframe();
    let _kernel_elf = kernel_elf::elf();
    info!("stacktrace: \n");
    while !stack_frame.is_null() {
        let ip = (*stack_frame).ip();
        info!("{} [0x{:016x}] ({}:{})", "symbol", ip, "file.rs", 0);
        stack_frame = (*stack_frame).next();
    }
}

#[panic_handler]
unsafe fn panic(info: &PanicInfo) -> ! {
    interrupt::disable();
    error!("[PANIC] {info}\n");
    unwind_stack();
    loop {
        interrupt::halt();
    }
}
