#![no_std]
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
extern crate cfg_if;

#[allow(unused_imports)]
#[macro_use]
extern crate util;

pub mod klog;

mod panic;
mod tty;

const VERSION_PATCH: usize = 0;
const VERSION_MINOR: usize = 0;
const VERSION_MAJOR: usize = 0;

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    info!("Acorn kernel v. {VERSION_MAJOR}.{VERSION_MINOR}.{VERSION_PATCH}");

    tty::run();

    error!("Kernel halted...");
    loop {
        util::ei();
        util::halt()
    }
}
