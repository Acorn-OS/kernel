use limine::limine_tag;

pub type MMap = limine::LimineMemmapResponse;

#[limine_tag]
static LIMINE_KERNEL_ADDRESS: limine::LimineKernelAddressRequest =
    limine::LimineKernelAddressRequest::new(0);

#[limine_tag]
static LIMINE_MMAP: limine::LimineMemmapRequest = limine::LimineMemmapRequest::new(0);

#[limine_tag]
static LIMINE_RDSP: limine::LimineRsdpRequest = limine::LimineRsdpRequest::new(0);

pub unsafe fn kernel_address() -> &'static mut limine::LimineKernelAddressResponse {
    LIMINE_KERNEL_ADDRESS.get_response().get_mut().unwrap()
}

pub unsafe fn rsdp() -> &'static mut limine::LimineRsdpResponse {
    LIMINE_RDSP.get_response().get_mut().unwrap()
}

pub unsafe fn get_mmap() -> &'static mut MMap {
    LIMINE_MMAP.get_response().get_mut().unwrap()
}

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

pub unsafe extern "C" fn kernel_early() -> ! {
    unsafe { call_init_arrays() };
    crate::main()
}
