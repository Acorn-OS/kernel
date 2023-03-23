mod limine;

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
unsafe extern "C" fn _start() -> ! {
    crate::logging::init();
    let ptr = &BOOT_INFO_DATA as *const _ as *mut BootInfo;
    let boot_info = &mut *ptr;
    let kernel_address = limine::kernel_address();
    boot_info.kernel_phys_base = kernel_address.physical_base;
    boot_info.kernel_virt_base = kernel_address.virtual_base;
    info!(
        "kernel mapped from phys [{:016X}] to virt [{:016X}]",
        BOOT_INFO.kernel_phys_base, BOOT_INFO.kernel_virt_base
    );
    unsafe { call_init_arrays() };
    crate::main()
}

pub struct BootInfo {
    pub kernel_phys_base: u64,
    pub kernel_virt_base: u64,
}

#[used]
static mut BOOT_INFO_DATA: BootInfo = unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
pub static BOOT_INFO: &BootInfo = unsafe { &BOOT_INFO_DATA };
