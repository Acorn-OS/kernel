use crate::boot::BootInfo;

#[repr(C, packed)]
pub struct RDSP {
    pub signature: [char; 8],
    pub checksum: u8,
    pub oem_id: [char; 8],
    pub revision: u8,
    pub rsdt_adr: u32,
    pub length: u32,
    pub xsdt_adr: u64,
    pub extended_checksum: u8,
    reserved: [u8; 3],
}

pub fn get(boot_info: &BootInfo) -> &'static RDSP {
    unsafe {
        &*(boot_info
            .rsdp
            .address
            .as_ptr()
            .expect("cannot get RDSP address!") as *const RDSP)
    }
}
