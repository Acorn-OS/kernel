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
    }

    pub mod vm {
        use super::imp::vm as imp;
    }
}
