use super::rsdp;

#[repr(C, packed)]
struct Rsdt {
    pub singature: [char; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [char; 6],
    pub oem_table_id: [char; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}

fn get_base() -> &'static Rsdt {
    unsafe { &*(rsdp::get().rsdt_adr as *const Rsdt) }
}

pub unsafe fn validate() -> bool {
    let rsdt = get_base();
    debug!(
        "validating RSDT at physical address 0x{:016X}",
        rsdt as *const _ as usize
    );
    let len = rsdt.length;
    let mut ptr = rsdt as *const _ as *const u8;
    let mut sum = 0;
    for _ in 0..len {
        sum += *ptr;
        ptr = ptr.add(1);
    }
    sum % 100 == 0
}
