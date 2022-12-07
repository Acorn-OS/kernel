#![no_std]
#![warn(missing_docs)]

extern crate alloc;

#[macro_use]
extern crate util;

#[macro_use]
extern crate static_assertions;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        mod amd64;
        use amd64 as imp;
    } else{
        const_assert!(false, "Unsupported target architecture.")
    }
}

/// Interrupt request.
pub mod irq {
    use crate::imp::irq;

    pub enum IRQ {}

    /// Enables a single IRQ.
    pub fn irq_en(irq: IRQ) {
        irq::irq_en(irq)
    }

    /// DIsambles a single IRQ.
    pub fn irq_di(irq: IRQ) {
        irq::irq_di(irq)
    }

    /// Check if an IRQ is enabled.
    pub fn irq_is_en(irq: IRQ) -> bool {
        irq::irq_is_en(irq)
    }

    /// Enables all IRQs.
    pub fn irq_en_all() {
        irq::irq_en_all()
    }

    /// Disables all IRQs.
    pub fn irq_di_all() {
        irq::irq_di_all()
    }
}

/// Virtual memory.
pub mod vm {
    use crate::imp::vm;

    /// Maps virtual memory.
    pub fn map() {
        vm::map(virt, phys, size)
    }

    /// Unmaps virtual memory.
    pub fn unmap() {
        vm::map(virt, phys, size)
    }
}

/// Framebuffer.
pub mod fb {
    /// Set cursor.
    pub fn cset() {}

    /// Clear framebuffer.
    pub fn clr() {}

    /// Get framebuffer dimensions.
    pub fn dim() {}

    /// Put a char onto the framebuffer.
    pub fn putc() {}

    /// Put a string onto the framebuffer.
    pub fn puts() {}
}
