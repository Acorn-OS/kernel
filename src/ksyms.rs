#![allow(dead_code)]

extern "C" {
    static kernel_virt_adr: u8;
    static kernel_virt_end: u8;
    static kernel_free_mem_adr: u8;
    static base_page_table: u8;
    static kernel_free_mem_len: u8;
}

pub fn free_mem_adr() -> usize {
    unsafe { &kernel_free_mem_adr as *const _ as usize }
}

pub fn free_mem_len() -> usize {
    unsafe { &kernel_free_mem_len as *const _ as usize }
}

pub fn virt_adr_start() -> usize {
    unsafe { &kernel_virt_adr as *const _ as usize }
}

pub fn virt_adr_end() -> usize {
    unsafe { &kernel_virt_end as *const _ as usize }
}

pub fn virt_len() -> usize {
    1 + virt_adr_end() - virt_adr_start()
}

/// root page table.
pub fn root_pt() -> usize {
    unsafe { &base_page_table as *const _ as usize }
}
