use crate::arch::{self, interrupt};
use crate::mm::pmm;
use crate::{kernel_elf, logging};
use limine::limine_tag;

pub type MMap = limine::LimineMemmapResponse;

#[limine_tag]
static LIMINE_KERNEL_ADDRESS: limine::LimineKernelAddressRequest =
    limine::LimineKernelAddressRequest::new(u64::MAX);

#[limine_tag]
static LIMINE_MMAP: limine::LimineMemmapRequest = limine::LimineMemmapRequest::new(u64::MAX);

#[limine_tag]
static LIMINE_RDSP: limine::LimineRsdpRequest = limine::LimineRsdpRequest::new(u64::MAX);

#[limine_tag]
static LIMINE_HHDM: limine::LimineHhdmRequest = limine::LimineHhdmRequest::new(u64::MAX);

#[limine_tag]
static LIMINE_MODULES: limine::LimineModuleRequest = limine::LimineModuleRequest::new(u64::MAX);

static LIMINE_KERNEL_FILE: limine::LimineKernelFileRequest =
    limine::LimineKernelFileRequest::new(u64::MAX);

pub struct BootInfo {
    pub kernel_address: &'static limine::LimineKernelAddressResponse,
    pub rsdp: &'static limine::LimineRsdpResponse,
    pub mmap: &'static mut MMap,
    pub hhdm: &'static limine::LimineHhdmResponse,
    pub modules: &'static limine::LimineModuleResponse,
    pub file: &'static limine::LimineKernelFileResponse,
}

impl BootInfo {
    pub unsafe fn get() -> BootInfo {
        Self {
            kernel_address: LIMINE_KERNEL_ADDRESS.get_response().get().unwrap(),
            rsdp: LIMINE_RDSP.get_response().get().unwrap(),
            mmap: LIMINE_MMAP.get_response().get_mut().unwrap(),
            hhdm: LIMINE_HHDM.get_response().get().unwrap(),
            modules: LIMINE_MODULES.get_response().get().unwrap(),
            file: LIMINE_KERNEL_FILE.get_response().get().unwrap(),
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
    trace!("initializing pmm");
    pmm::init(&mut boot_info);
    debug!(
        "initialized hhdm mapping with form 0x{:016x} to 0x{:016x}",
        pmm::hhdm_base(),
        pmm::hhdm_base() + pmm::hhdm_len() as u64
    );
    trace!("initializing arch specific code");
    arch::arch_init(&mut boot_info);
    trace!("initializing kernel elf");
    kernel_elf::init(&boot_info);
    trace!("calling init arrays");
    call_init_arrays();
    trace!("starting kernel processes");
    crate::init::run(boot_info);
}
