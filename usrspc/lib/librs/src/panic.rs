use core::panic::PanicInfo;

#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
    syscall::kprint("panicked!");
    loop {}
}
