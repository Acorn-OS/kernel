#![no_std]
#![no_main]

mod panic;

#[no_mangle]
extern "C" fn _start() -> ! {
    std::test_print("test application! To remove!");
    loop {}
}
