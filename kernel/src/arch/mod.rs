cfg_if! {
    if #[cfg(target_arch = "x86_64")]{
        mod amd64;
        use amd64 as imp;
    } else {
        static_assert!(false, "unsupported target")
    }
}

pub mod serial {
    use super::imp;

    pub mod uart {
        use super::imp::serial::uart;

        pub fn putb(b: u8) {
            uart::putb(b)
        }

        pub fn putc(c: char) {
            putb(c as u8)
        }

        pub fn puts(s: &str) {
            for c in s.chars() {
                putc(c);
            }
        }
    }
}

pub mod vm {
    use super::imp::vm;

    pub use vm::AllocSize;
    pub use vm::PageMap;
}

pub mod cpu {
    use super::imp::cpu;

    pub use cpu::Core;
}

pub mod fb {
    use super::imp::fb;

    pub const WIDTH: usize = fb::WIDTH;
    pub const HEIGHT: usize = fb::HEIGHT;

    pub type Cursor = fb::Cursor;

    pub fn putb(pos: Cursor, b: u8) {
        fb::putb(pos, b)
    }
}
