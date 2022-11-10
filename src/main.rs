// Temporary
#![allow(dead_code)]
// End of temporary
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[allow(unused_imports)]
#[macro_use]
extern crate core;

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[allow(unused_imports)]
#[macro_use]
extern crate log;

#[allow(unused_imports)]
#[macro_use]
extern crate static_assertions;

#[allow(unused_imports)]
#[macro_use]
mod util;

mod arch;
mod drivers;
mod fb;
mod klog;
mod ksyms;
mod mm;
mod tty;

const VERSION_PATCH: usize = 0;
const VERSION_MINOR: usize = 0;
const VERSION_MAJOR: usize = 0;

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // Initializes com drivers for serial printing.
    drivers::com::init();

    klog::init();

    mm::vmalloc::init();
    mm::paging::init();
    drivers::init();

    fb::clear();
    info!("Acorn kernel v. {VERSION_MAJOR}.{VERSION_MINOR}.{VERSION_PATCH}");

    info!("Looping...");
    loop {
        util::ei();
        util::halt()
    }
}

#[panic_handler]
pub unsafe fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    let msg = if let Some(msg) = info.message() {
        *msg
    } else {
        format_args!("(UNAVAILABLE)")
    };
    let (file, line, column) = if let Some(msg) = info.location() {
        (msg.file(), msg.line(), msg.column())
    } else {
        ("(UNAVAILABLE)", 0, 0)
    };
    info!("panic! in {} at {}:{}\n\r{}", file, line, column, msg);
    loop {
        util::di();
        util::halt()
    }
}
