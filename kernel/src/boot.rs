use crate::arch::{self, interrupt};
use crate::logging;
use crate::mm::{heap, pmm, vmm};
use limine::limine_tag;

pub type MMap = limine::LimineMemmapResponse;

#[limine_tag]
static LIMINE_KERNEL_ADDRESS: limine::LimineKernelAddressRequest =
    limine::LimineKernelAddressRequest::new(0);

#[limine_tag]
static LIMINE_MMAP: limine::LimineMemmapRequest = limine::LimineMemmapRequest::new(u64::MAX);

#[limine_tag]
static LIMINE_RDSP: limine::LimineRsdpRequest = limine::LimineRsdpRequest::new(u64::MAX);

#[limine_tag]
static LIMINE_HHDM: limine::LimineHhdmRequest = limine::LimineHhdmRequest::new(u64::MAX);

#[limine_tag]
static LIMINE_MODULES: limine::LimineModuleRequest = limine::LimineModuleRequest::new(u64::MAX);

pub struct BootInfo {
    pub kernel_address: &'static limine::LimineKernelAddressResponse,
    pub rsdp: &'static limine::LimineRsdpResponse,
    pub mmap: &'static mut MMap,
    pub hhdm: &'static limine::LimineHhdmResponse,
    pub modules: &'static limine::LimineModuleResponse,
}

impl BootInfo {
    pub unsafe fn get() -> BootInfo {
        Self {
            kernel_address: LIMINE_KERNEL_ADDRESS.get_response().get().unwrap(),
            rsdp: LIMINE_RDSP.get_response().get().unwrap(),
            mmap: LIMINE_MMAP.get_response().get_mut().unwrap(),
            hhdm: LIMINE_HHDM.get_response().get().unwrap(),
            modules: LIMINE_MODULES.get_response().get().unwrap(),
        }
    }
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

#[no_mangle]
pub unsafe extern "C" fn kernel_early() -> ! {
    interrupt::disable();
    logging::init();
    let mut boot_info = BootInfo::get();
    pmm::init(&mut boot_info);
    trace!(
        "initialized physical memory management with '{}' pages",
        pmm::page_cnt()
    );
    trace!("initializing heap");
    arch::arch_init(&mut boot_info);
    heap::init();
    vmm::init(&boot_info);
    call_init_arrays();
    crate::main(boot_info)
}
