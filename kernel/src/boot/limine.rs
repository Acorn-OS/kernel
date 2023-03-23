use core::ptr::null_mut;

mod req {
    use super::*;

    #[repr(C)]
    pub struct KernelAddressReq {
        pub id: [u64; 4],
        pub revision: u64,
        pub kernel_address: *mut KernelAddress,
    }
}

const fn create_id(idlo: u64, idhi: u64) -> [u64; 4] {
    [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b, idlo, idhi]
}

pub struct KernelAddress {
    pub revision: u64,
    pub physical_base: u64,
    pub virtual_base: u64,
}

#[used]
static mut KERNEL_ADDRESS_REQ: req::KernelAddressReq = req::KernelAddressReq {
    id: create_id(0x71ba76863cc55f63, 0xb2644a48c516a487),
    revision: 0,
    kernel_address: null_mut(),
};

#[used]
#[link_section = ".limine_reqs"]
static mut KERNEL_ADDRESS_REQ_PTR: *mut req::KernelAddressReq =
    unsafe { &KERNEL_ADDRESS_REQ as *const _ as *mut _ };

pub fn kernel_address() -> &'static KernelAddress {
    unsafe { &*KERNEL_ADDRESS_REQ.kernel_address }
}
