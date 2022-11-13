// Temporary
#![allow(dead_code)]
// End of temporary
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(c_unwind)]
#![feature(lang_items)]
#![feature(panic_runtime)]

#[allow(unused_imports)]
#[macro_use]
extern crate core;

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[allow(unused_imports)]
#[macro_use]
extern crate static_assertions;

#[allow(unused_imports)]
#[macro_use]
extern crate util;

mod boot;
mod drivers;
mod fb;
mod klog;
mod ksyms;
mod mm;
mod panic;
mod tty;

const VERSION_PATCH: usize = 0;
const VERSION_MINOR: usize = 0;
const VERSION_MAJOR: usize = 0;

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    info!("Acorn kernel v. {VERSION_MAJOR}.{VERSION_MINOR}.{VERSION_PATCH}");

    tty::run();

    error!("Kernel halted...");
    loop {
        util::ei();
        util::halt()
    }
}
