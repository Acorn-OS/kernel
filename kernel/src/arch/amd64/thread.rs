use super::cpu::Core;
use super::msr;
use crate::process::thread::{Thread, ThreadPtr};
use crate::util::adr::VirtAdr;
use core::arch::asm;
use core::ptr::null_mut;

#[derive(Debug)]
#[repr(C, align(8))]
pub struct ArchThread {
    ptr: u64,
    cur_core: *mut Core,
}

impl ArchThread {
    pub fn new() -> Self {
        Self {
            ptr: 0,
            cur_core: null_mut(),
        }
    }

    pub unsafe fn init(thread: &mut Thread) {
        let arch_ptr = thread.as_ptr();
        let arch = thread.arch_mut();
        arch.ptr = arch_ptr.as_adr().adr();
    }

    pub(super) fn core_ptr(&self) -> *mut Core {
        self.cur_core
    }
}

pub fn get_gs_base() -> VirtAdr {
    VirtAdr::new(msr::get(msr::GS_BASE))
}

pub fn get_kernel_gs_base() -> VirtAdr {
    VirtAdr::new(msr::get(msr::KERNEL_GS_BASE))
}

unsafe fn set_kernel_gs_base(ptr: *const Thread) {
    msr::set(msr::KERNEL_GS_BASE, ptr as u64);
}

unsafe fn set_gs_base(ptr: *const Thread) {
    msr::set(msr::GS_BASE, ptr as u64);
}

pub fn cur_thread() -> ThreadPtr {
    let ptr: u64;
    unsafe {
        asm!("mov rax, gs:[0]", out("rax") ptr);
        ThreadPtr::from_ptr(ptr as *mut Thread)
    }
}

pub unsafe fn set_thread(thread: &mut Thread) {
    let core_ptr = super::cpu::get_core();
    debug_assert!(!core_ptr.is_null());
    thread.arch_mut().cur_core = core_ptr;
    set_gs_base(thread.as_ptr().as_ptr());
    set_kernel_gs_base(null_mut());
}
