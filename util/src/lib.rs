#![no_std]

pub extern crate spin;

#[macro_export]
macro_rules! once {
    { $($tt:tt)* } => {
        {
            static __ONCE__: $crate::spin::Once = $crate::spin::Once::new();
            *__ONCE__.call_once(|| { $($tt)* })
        }
    };
}

#[macro_export]
macro_rules! guard {
    ($($tt:tt)*) => {{
        static __GUARD__: $crate::spin::Mutex<()> = $crate::spin::Mutex::new(());
        let _lock = __GUARD__.lock();
        { $($tt)* }
    }};
}

/// Delays roughly `amount` of cycles.
#[inline(always)]
pub fn delay(amount: u64) {
    ::core::hint::black_box(for _ in 0..amount {});
}
