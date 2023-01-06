#![no_std]
#![no_main]
#![feature(int_roundings)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

extern crate alloc;
extern crate core;

#[macro_use]
extern crate log;
#[macro_use]
extern crate static_assertions;
extern crate spin;
#[macro_use]
extern crate proc_bitfield;

#[macro_use]
mod klog;
mod arch;
mod math;
mod mm;
mod panic;

#[macro_export]
macro_rules! once {
    { $($tt:tt)* } => {
        {
            static __ONCE__: ::spin::Once = ::spin::Once::new();
            *__ONCE__.call_once(|| { $($tt)* })
        }
    };
}

/// Delays roughly `amount` of cycles.
#[inline(always)]
fn delay(amount: u64) {
    ::core::hint::black_box(for _ in 0..amount {});
}

fn kmain() -> ! {
    unsafe { arch::serial::init() };
    klog::init();
    trace_init!("logging");

    mm::init();
    trace_init!("mm");

    unsafe {
        arch::fb::init();
        trace_init!("fb");
    }

    info!("Hello!");
    loop {}
}
