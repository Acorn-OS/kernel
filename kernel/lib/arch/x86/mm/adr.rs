use crate::arch::mm::vptr;

pub const KVIRT_BEG: vptr = 0xFFFFFFFF80000000;
pub const KVIRT_END: vptr = 0xFFFFFFFFFFFFFFFF;
