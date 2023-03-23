use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("[PANIC] {info}");
    loop {}
}
