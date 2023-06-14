pub mod adr;
pub mod locked;

/// Delays roughly `amount` of cycles.
#[inline(always)]
pub fn delay(amount: u64) {
    ::core::hint::black_box(for _ in 0..amount {});
}
