//! COM serial hardware.

use amd64::cpu::io::{in8, out8};
use spin::Mutex;

struct COM(u16);

impl COM {
    fn init(&self) {
        let port_adr = self.0;
        out8(port_adr + 1, 0x00);
        out8(port_adr + 3, 0x80);
        out8(port_adr + 0, 0x03);
        out8(port_adr + 1, 0x00);
        out8(port_adr + 3, 0x03);
        out8(port_adr + 2, 0xC7);
        out8(port_adr + 4, 0x0B);
        out8(port_adr + 4, 0x1E);

        // Test connection.
        out8(port_adr, 0xEA);
        if in8(port_adr) != 0xEA {
            panic!("Unable to properly initialize COM port {}.", self.0)
        }
        out8(port_adr + 4, 0x0F);
    }

    fn putb(&self, b: u8) {
        out8(self.0, b);
    }

    fn putc(&self, c: char) {
        self.putb(c as u8)
    }

    fn puts(&self, s: &str) {
        for c in s.chars() {
            self.putc(c)
        }
    }
}

#[allow(missing_docs, dead_code)]
#[derive(Clone, Copy, Debug)]
enum Port {
    COM1,
    COM2,
    COM3,
    COM4,
    COM5,
    COM6,
    COM7,
    COM8,
}

static LOCKS: [Mutex<COM>; 8] = [
    Mutex::new(COM(0x3F8)),
    Mutex::new(COM(0x2F8)),
    Mutex::new(COM(0x3E8)),
    Mutex::new(COM(0x2E8)),
    Mutex::new(COM(0x5F8)),
    Mutex::new(COM(0x4F8)),
    Mutex::new(COM(0x5E8)),
    Mutex::new(COM(0x4E8)),
];

macro_rules! lock {
    ($port:expr) => {
        LOCKS[match $port {
            Port::COM1 => 0,
            Port::COM2 => 1,
            Port::COM3 => 2,
            Port::COM4 => 3,
            Port::COM5 => 4,
            Port::COM6 => 5,
            Port::COM7 => 6,
            Port::COM8 => 7,
        }]
        .lock()
    };
}

#[inline(always)]
pub fn init() {
    use Port::*;
    once! {
        lock!(COM1).init();
    }
}

#[allow(unused)]
#[inline(always)]
pub fn putb(b: u8) {
    lock!(Port::COM1).putb(b)
}

#[allow(unused)]
#[inline(always)]
pub fn putc(c: char) {
    lock!(Port::COM1).putc(c)
}

#[allow(unused)]
#[inline(always)]
pub fn puts(s: &str) {
    lock!(Port::COM1).puts(s)
}
