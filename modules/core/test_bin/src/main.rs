#![no_std]
#![no_main]

mod panic;

#[no_mangle]
extern "C" fn _start() -> ! {
    loop {}
}
