#![allow(dead_code)]

macro_rules! delay {
    ($amnt:expr) => {
        crate::util::delay($amnt)
    };
}

macro_rules! once {
    ($($tt:tt)*) => {
        {
            static __ONCE__: ::spin::Once = ::spin::Once::new();
            *__ONCE__.call_once(|| { $($tt)* })
        }
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
