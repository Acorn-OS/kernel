#![no_std]
#![no_main]

#[macro_use]
extern crate std;

mod panic;

#[no_mangle]
extern "C" fn _start() -> ! {
    println!("starting the ps2 driver");
    loop {}
}
