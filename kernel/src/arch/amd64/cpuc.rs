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
    internal_ptr: u64,
    pub(super) lapic_ptr: LApicPtr,
}

pub unsafe fn swap() {
    asm!("swapgs");
}

unsafe fn set_kernel_gs_base(ptr: *mut Core) {
    msr::set(msr::KERNEL_GS_BASE, ptr as u64);
}

unsafe fn set_gs_base(ptr: *mut Core) {
    msr::set(msr::GS_BASE, ptr as u64);
}

pub fn get_kernel() -> NonNull<Core> {
    let ptr: u64;
    unsafe { asm!("mov rax, gs:[0]", out("rax") ptr) }
    let ptr = ptr as *mut Core;
    debug_assert!(!ptr.is_null());
    unsafe { NonNull::new_unchecked(ptr) }
}

pub unsafe fn init_for_core() {
    trace!("initializing LAPIC");
    let lapic_ptr = lapic::create_local();
    trace!("setting up cpuc object");
    let core = pmm::alloc_pages(pages!(size_of::<Core>()));
    (core.virt().ptr() as *mut Core).write(Core {
        internal_ptr: core.virt().adr(),
        lapic_ptr,
    });
    set_kernel_gs_base(null_mut());
    set_gs_base(core.virt().ptr() as *mut _);
}
