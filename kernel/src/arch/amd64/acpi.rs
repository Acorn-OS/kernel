#[repr(C, packed)]
pub struct Madt {
    signature: u32,
    length: u32,
    revision: u8,
    checksum: u8,
    oemid: [u8; 6],
    oem_table_id: u64,
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
    local_apic_adr: u32,
    flags: u32,
}

pub fn parse() -> Madt {
    todo!()
}
