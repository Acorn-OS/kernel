use super::pmm::{self, Page};
use crate::boot::BootInfo;

pub const PAGE_SIZE: usize = pmm::PAGE_SIZE;
pub const PAGE_EXP: usize = pmm::PAGE_EXP;

static mut HHDM_SIZE: usize = 4 << 30;
static mut HHDM_BASE: u64 = 0;

pub fn to_virt(phys_page: *const Page) -> *mut Page {
    unsafe { (phys_page as u64 + HHDM_BASE) as *mut _ }
}

pub fn to_phys(virt_page: *mut Page) -> *const Page {
    unsafe { (virt_page as u64 - HHDM_BASE) as *const _ }
}

pub fn base() -> u64 {
    unsafe { HHDM_BASE }
}

pub fn size() -> usize {
    unsafe { HHDM_SIZE }
}

pub unsafe fn init(boot_info: &BootInfo) {
    HHDM_BASE = boot_info.hhdm.offset;
}
