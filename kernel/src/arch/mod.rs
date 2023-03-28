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

pub mod vmm {
    use super::imp::vmm;

    pub use vmm::AllocSize;
    pub use vmm::PageMap;

    pub fn new_kernel() -> *mut PageMap {
        unsafe { vmm::new_kernel() }
    }

    pub unsafe fn install(map: *mut PageMap) {
        vmm::install(map)
    }
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

    pub fn cursor() -> Cursor {
        fb::cursor()
    }

    pub fn set_cursor(cursor: Cursor) {
        fb::set_cursor(cursor)
    }

    pub fn putb(b: u8) {
        fb::putb(b)
    }

    pub fn putc(c: char) {
        putb(c as u8)
    }

    pub fn puts(s: &str) {
        for c in s.chars() {
            putc(c)
        }
    }
}
