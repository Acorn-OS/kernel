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
extern crate static_assertions;

#[allow(unused_imports)]
#[macro_use]
extern crate util;

extern crate unwinding;

mod boot;
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
    mm::vmalloc::init();
    drivers::init();

    fb::clear();
    info!("Acorn kernel v. {VERSION_MAJOR}.{VERSION_MINOR}.{VERSION_PATCH}");

    info!("Looping...");
    loop {
        util::ei();
        util::halt()
    }
}

pub unsafe fn stack_trace() {
    struct CallbackData {
        counter: usize,
    }
    extern "C" fn callback(
        unwind_ctx: &mut unwinding::abi::UnwindContext<'_>,
        arg: *mut core::ffi::c_void,
    ) -> unwinding::abi::UnwindReasonCode {
        let data = unsafe { &mut *(arg as *mut CallbackData) };
        data.counter += 1;
        if data.counter < 1 {
            debug!(
                "{:4} {:19X}\n",
                data.counter,
                unwinding::abi::_Unwind_GetIP(unwind_ctx)
            );
        }
        unwinding::abi::UnwindReasonCode::NO_REASON
    }
    let mut data = CallbackData { counter: 0 };
    unwinding::abi::_Unwind_Backtrace(callback, &mut data as *mut _ as _);
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
    stack_trace();
    unwinding::panic::begin_panic(alloc::boxed::Box::new(()));
    loop {
        util::di();
        util::halt()
    }
}
