#![no_std]
#![no_main]
#![feature(panic_info_message)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate alloc;

mod boot;
mod panic;
mod tty;

fn main() -> ! {
    info!("AcornOS");
    tty::run();
    error!("hanging kernel ungracefully...");
    let mut string = alloc::string::String::new();
    info!("test0: {string}");
    string.push_str("hello world!");
    info!("test1: {string}");
    string.push_str(" yeahhhhhhhhhhhhhhhhh baby!");
    info!("test2: {string}");
    info!(
        "test3: {}",
        format!("{string} THAT IS WHAT I AM TALKING ABOUT!!!!")
    );
    loop {}
}
