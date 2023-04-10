use crate::arch::imp::port::out8;

pub struct Uart(u16);

impl Uart {
    pub fn putb(&self, b: u8) {
        out8(self.0, b)
    }
}

static UARTS: &[Uart] = &[
    // COM1
    Uart(0x3F8),
];

pub static DEFAULT_UART: &Uart = &UARTS[0];

pub fn putb(b: u8) {
    DEFAULT_UART.putb(b);
}

pub fn putc(c: char) {
    putb(c as u8)
}

pub fn puts(s: &str) {
    for c in s.chars() {
        putc(c);
    }
}
