/// Delays roughly `amount` of cycles.
#[inline(always)]
pub fn delay(amount: u64) {
    ::core::hint::black_box(for _ in 0..amount {});
}

pub fn irq_di() {
    unsafe { core::arch::asm!("cli") };
}

pub fn irq_en() {
    unsafe { core::arch::asm!("sti") }
}

pub fn halt() {
    unsafe {
        core::arch::asm!("hlt");
    }
}
