use core::ptr::null_mut;

const fn new_id(idlo: u64, idhi: u64) -> [u64; 4] {
    [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b, idlo, idhi]
}

mod req {
    use super::*;

    #[repr(C)]
    pub struct KernelAddressReq {
        pub id: [u64; 4],
        pub revision: u64,
        pub response: *mut KernelAddress,
    }

    pub struct MMapReq {
        pub id: [u64; 4],
        pub revision: u64,
        pub response: *mut MMap,
    }
}

#[repr(C)]
pub struct KernelAddress {
    pub revision: u64,
    pub physical_base: u64,
    pub virtual_base: u64,
}

#[repr(C)]
pub struct MMapEntry {
    pub base: u64,
    pub len: u64,
    pub ty: u64,
}

impl MMapEntry {
    pub const USABLE: u64 = 0;
    pub const RESERVED: u64 = 1;
    pub const ACPI_RECLAIMABLE: u64 = 2;
    pub const ACPI_NVS: u64 = 3;
    pub const BAD_MEMORY: u64 = 4;
    pub const BOOTLOADER_RECLAIMABLE: u64 = 5;
    pub const KERNEL_AND_MODULES: u64 = 6;
    pub const FRAMEBUFFER: u64 = 7;
}

#[repr(C)]
pub struct MMap {
    pub revision: u64,
    pub entry_count: u64,
    pub entries: *mut *mut MMapEntry,
}

unsafe impl Send for MMap {}
unsafe impl Sync for MMap {}

#[used]
static mut KERNEL_ADDRESS_REQ: req::KernelAddressReq = req::KernelAddressReq {
    id: new_id(0x71ba76863cc55f63, 0xb2644a48c516a487),
    revision: u64::MAX,
    response: null_mut(),
};

#[used]
static mut MMAP_REQ: req::MMapReq = req::MMapReq {
    id: new_id(0x67cf3d9d378a806f, 0xe304acdfc50c3c62),
    revision: u64::MAX,
    response: null_mut(),
};

#[used]
#[link_section = ".limine_reqs"]
static mut PTRS: [*mut (); 2] = unsafe {
    [
        &KERNEL_ADDRESS_REQ as *const _ as *mut _,
        &MMAP_REQ as *const _ as *mut _,
    ]
};

pub fn kernel_address() -> &'static KernelAddress {
    unsafe { &*KERNEL_ADDRESS_REQ.response }
}

pub fn mmap() -> &'static MMap {
    unsafe { &*MMAP_REQ.response }
}
