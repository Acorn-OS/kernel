use limine::limine_tag;

pub type MMap = limine::LimineMemmapResponse;

#[limine_tag]
static LIMINE_KERNEL_ADDRESS: limine::LimineKernelAddressRequest =
    limine::LimineKernelAddressRequest::new(0);

#[limine_tag]
static LIMINE_MMAP: limine::LimineMemmapRequest = limine::LimineMemmapRequest::new(0);

#[limine_tag]
static LIMINE_RDSP: limine::LimineRsdpRequest = limine::LimineRsdpRequest::new(0);

#[limine_tag]
static LIMINE_HHDM: limine::LimineHhdmRequest = limine::LimineHhdmRequest::new(0);

pub struct BootInfo {
    pub kernel_address: &'static limine::LimineKernelAddressResponse,
    pub rsdp: &'static limine::LimineRsdpResponse,
    pub mmap: &'static mut MMap,
    pub hhdm: &'static limine::LimineHhdmResponse,
}

impl BootInfo {
    pub unsafe fn get() -> BootInfo {
        Self {
            kernel_address: LIMINE_KERNEL_ADDRESS.get_response().get().unwrap(),
            rsdp: LIMINE_RDSP.get_response().get().unwrap(),
            mmap: LIMINE_MMAP.get_response().get_mut().unwrap(),
            hhdm: LIMINE_HHDM.get_response().get().unwrap(),
        }
    }
}

pub unsafe fn info() -> BootInfo {
    BootInfo::get()
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
