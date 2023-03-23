use crate::arch::imp::port::out8;
use crate::arch::serial::Serial;

pub struct UART(u16);

impl Serial for UART {
    fn putb(&self, b: u8) {
        out8(self.0, b)
    }
}

static UARTS: &[UART] = &[
    // COM1
    UART(0x3F8),
];

pub static DEFAULT_UART: &UART = &UARTS[0];
