use crate::boot::limine;

#[repr(C, packed)]
pub struct Rdsp {
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

pub fn get() -> &'static Rdsp {
    debug!("RDSP address: {:016X}", limine::rsdp().address as usize);
    unsafe { &*(limine::rsdp().address as *const Rdsp) }
}
