cfg_if! {
    if #[cfg(target_arch = "x86_64")]{
        mod amd64;
        use amd64 as imp;
    } else {
        static_assert!(false, "unsupported target")
    }
}

pub mod serial {
    pub trait Serial {
        fn putb(&self, b: u8);

        fn putc(&self, c: char) {
            self.putb(c as u8)
        }

        fn puts(&self, s: &str) {
            for c in s.chars() {
                self.putc(c)
            }
        }
    }

    pub mod uart {
        use super::super::imp::serial::uart;
        use super::Serial;

        pub type UART = uart::UART;
        assert_impl_all!(UART: Serial);

        static DEFAULT_UART: &UART = uart::DEFAULT_UART;

        pub fn putc(c: char) {
            Serial::putc(DEFAULT_UART, c)
        }

        pub fn puts(s: &str) {
            Serial::puts(DEFAULT_UART, s)
        }
    }
}
