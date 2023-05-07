use crate::boot::BootInfo;
use crate::mm::pmm;

#[repr(C, packed)]
struct Rsdp {
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

fn get_rsdp(boot_info: &BootInfo) -> &'static Rsdp {
    unsafe {
        &*(boot_info
            .rsdp
            .address
            .as_ptr()
            .expect("cannot get RDSP address!") as *const Rsdp)
    }
}

#[repr(C, packed)]
pub struct Rsdt {
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

fn get_base(rsdp: &Rsdp) -> &'static Rsdt {
    unsafe { &*(pmm::phys_to_hhdm(rsdp.rsdt_adr as u64) as *const Rsdt) }
}

unsafe fn validate(rsdt: &Rsdt) -> bool {
    debug!(
        "validating RSDT at physical address 0x{:016x}",
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

pub unsafe fn init(boot_info: &BootInfo) -> &'static Rsdt {
    let rsdp = get_rsdp(boot_info);
    let rsdt = get_base(rsdp);
    assert!(validate(rsdt), "the RSDT could not be validated");
    rsdt
}
