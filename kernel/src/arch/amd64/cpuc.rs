use super::apic::lapic;
use super::apic::LApicPtr;
use super::msr;
use crate::mm::pmm;
use core::arch::asm;
use core::mem::size_of;
use core::ptr::null_mut;
use core::ptr::NonNull;

#[repr(C)]
pub struct Core {
    pub(super) lapic_ptr: LApicPtr,
}

pub unsafe fn swap() {
    asm!("swapgs");
}

pub unsafe fn set_kernel_gs_base(ptr: NonNull<Core>) {
    msr::set(msr::KERNEL_GS_BASE, ptr.addr().get() as u64);
}

pub unsafe fn set_gs_base(ptr: *mut Core) {
    msr::set(msr::GS_BASE, ptr as u64);
}

pub fn get_kernel() -> NonNull<Core> {
    let ptr = msr::get(msr::KERNEL_GS_BASE) as *mut Core;
    debug_assert!(!ptr.is_null());
    unsafe { NonNull::new_unchecked(ptr) }
}

pub unsafe fn init_for_core() {
    trace!("initializing LAPIC");
    let lapic_ptr = lapic::create_local();
    trace!("setting up cpuc object");
    let core = pmm::alloc_pages(pages!(size_of::<Core>()));
    core.as_virt_ptr::<Core>().write(Core { lapic_ptr });
    set_kernel_gs_base(NonNull::new_unchecked(core.as_virt_ptr()));
    set_gs_base(null_mut());
}
