#![no_std]
#![no_main]

mod panic;

#[no_mangle]
extern "C" fn _start() -> ! {
    std::test_print("ps2 driver booting!");
    loop {}
}
