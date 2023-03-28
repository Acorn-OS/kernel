pub mod limine;

use crate::arch::vmm;
use crate::logging;
use crate::mm::{heap, pmm};
use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));

/// Initialize all constructor functions
unsafe fn call_init_arrays() {
    extern "C" {
        static __init_array_beg: u8;
        static __init_array_end: u8;
    }
    let mut beg = &__init_array_beg as *const _ as *const extern "C" fn();
    let end = &__init_array_end as *const _ as *const extern "C" fn();
    while beg < end {
        (*beg)();
        beg = beg.add(1);
    }
}

#[no_mangle]
unsafe extern "C" fn kernel_early() -> ! {
    pmm::init();
    let vmm_map = vmm::new_kernel();
    vmm::install(vmm_map);
    logging::init();
    heap::init();
    unsafe { call_init_arrays() };
    crate::main()
}
