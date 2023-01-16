#![no_std]
#![no_main]
#![feature(int_roundings)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

#[macro_use]
extern crate alloc;
extern crate core;

#[macro_use]
pub extern crate log;

#[macro_use]
extern crate static_assertions;
extern crate spin;
#[macro_use]
extern crate proc_bitfield;

#[macro_use]
pub mod arch;
pub mod mm;

mod klog;
mod math;
mod panic;
mod tty;

macro_rules! once {
    { $($tt:tt)* } => {
        {
            static __ONCE__: ::spin::Once = ::spin::Once::new();
            *__ONCE__.call_once(|| { $($tt)* })
        }
    };
}
pub(crate) use once;

/// Delays roughly `amount` of cycles.
#[inline(always)]
fn delay(amount: u64) {
    ::core::hint::black_box(for _ in 0..amount {});
}

fn kmain() -> ! {
    log::info!("Welcome! AcornOS");
    tty::run();
    loop {}
}
