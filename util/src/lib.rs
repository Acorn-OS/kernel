#![no_std]
#![allow(dead_code)]

extern crate self as util;

pub mod logging;

#[macro_export]
macro_rules! once {
    ($($tt:tt)*) => {
        {
            static __ONCE__: ::spin::Once = ::spin::Once::new();
            *__ONCE__.call_once(|| { $($tt)* })
        }
    };
}

#[macro_export]
macro_rules! info {
    ($($tt:tt)*) => {
        ::util::logging::log::info!($($tt)*)
    };
}

#[macro_export]
macro_rules! warn {
    ($($tt:tt)*) => {
        ::util::logging::log::warn!($($tt)*)
    };
}

#[macro_export]
macro_rules! error {
    ($($tt:tt)*) => {
        ::util::logging::log::error!($($tt)*)
    };
}

#[macro_export]
macro_rules! log {
    ($($tt:tt)*) => {
        ::util::logging::log::log!($($tt)*)
    };
}

#[macro_export]
macro_rules! trace {
    ($($tt:tt)*) => {
        ::util::logging::log::trace!($($tt)*)
    };
}

#[macro_export]
macro_rules! debug {
    ($($tt:tt)*) => {
        ::util::logging::log::debug!($($tt)*)
    };
}

/// Delays roughly `amount` of cycles.
#[inline(always)]
pub fn delay(amount: u64) {
    ::core::hint::black_box(for _ in 0..amount {});
}

/// Halts CPU execution.
#[inline(always)]
pub fn halt() {
    unsafe { ::core::arch::asm!("hlt") }
}

/// Disables interrupts.
pub fn di() {
    unsafe { ::core::arch::asm!("cli") }
}

/// Enables interrupts.
pub fn ei() {
    unsafe { ::core::arch::asm!("sti") }
}
