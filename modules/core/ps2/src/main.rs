#![no_std]
#![no_main]

#[macro_use]
extern crate std;

#[no_mangle]
extern "C" fn _start() -> ! {
    println!("starting the ps2 driver");
    let mut string = std::string::String::new();
    for i in 0..10 {
        string.push_str(&format!("[{i}] "));
    }
    println!("constructed string: {string}");
    loop {}
}
