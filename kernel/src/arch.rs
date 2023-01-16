mod x86;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86_64")]{
        use x86 as imp;
    } else{
        const_assert!(false, "invalid arch")
    }
}

pub mod mm {
    use crate::arch::imp::mm as imp;

    /// Virtual address pointer.
    #[allow(non_camel_case_types)]
    pub type vptr = imp::vptr;
    /// Physical address pointer.
    #[allow(non_camel_case_types)]
    pub type pptr = imp::pptr;

    pub mod adr {
        use super::imp::adr as imp;

        use super::vptr;

        pub const KVIRT_BEG: vptr = imp::KVIRT_BEG;
        pub const KVIRT_END: vptr = imp::KVIRT_END;
    }

    pub mod vm {
        use super::imp::vm as imp;

        use super::{pptr, vptr};

        pub fn map(virt: vptr, phys: pptr) {
            imp::map(virt, phys)
        }

        pub fn unmap(virt: vptr) {
            imp::unmap(virt)
        }
    }
}

pub mod fb {
    use super::imp::fb as imp;

    use imp::Color;

    pub fn puts(s: &str) {
        imp::puts(s, Color::WHITE);
    }

    pub fn putlns(s: &str) {
        imp::puts(&format!("{s}\n\r"), Color::WHITE);
    }

    pub fn clear() {
        imp::clear()
    }
}

pub fn log(s: &str) {
    imp::log(s)
}
