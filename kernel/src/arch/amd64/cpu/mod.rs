pub mod ctrl_regs;

use super::apic::lapic;
use super::apic::LApicPtr;
use crate::mm::pmm;
use core::mem::size_of;
use core::ptr::null_mut;

static mut CUR_CORE: *mut Core = null_mut();

#[repr(C)]
pub struct Core {
    internal_ptr: u64,
    pub(super) lapic_ptr: LApicPtr,
}

pub(super) unsafe fn init_core() {
    trace!("initializing LAPIC");
    let lapic_ptr = lapic::create_local();
    trace!("setting up core local object");
    let core = pmm::alloc_pages(pages!(size_of::<Core>()));
    (core.virt().ptr() as *mut Core).write(Core {
        internal_ptr: core.virt().adr(),
        lapic_ptr,
    });
    CUR_CORE = core.virt().ptr() as *mut _;
}

pub(super) fn get_core() -> *mut Core {
    unsafe { CUR_CORE }
}
