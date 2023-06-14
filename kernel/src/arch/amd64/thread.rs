use super::cpu::Core;
use super::msr;
use crate::arch::interrupt::StackFrame;
use crate::mm::heap;
use crate::process::thread::ThreadId;
use crate::process::Process;
use crate::util::adr::VirtAdr;
use crate::util::locked::{Lock, LockGuard, Locked};
use alloc::boxed::Box;
use core::alloc::Layout;
use core::arch::asm;
use core::cell::UnsafeCell;
use core::ptr::{null_mut, NonNull};

#[derive(Debug)]
#[repr(C, align(8))]
pub struct ArchThreadInner {
    self_ptr: u64,
    pub(super) id: ThreadId,
    pub(super) proc: *mut Process,
    pub(super) stackframe: Box<StackFrame>,
    pub(super) stack: *mut u8,
    kernel_thread: bool,
    cur_core: *mut Core,
}

impl ArchThreadInner {
    #[inline]
    pub unsafe fn set_stackframe(&mut self, stackframe: StackFrame) {
        *self.stackframe = stackframe
    }

    #[inline]
    pub fn get_stackframe(&self) -> StackFrame {
        (*self.stackframe).clone()
    }

    #[inline]
    pub fn core_ref(&self) -> Option<&'static Core> {
        if self.cur_core.is_null() {
            None
        } else {
            unsafe { Some(&*self.cur_core) }
        }
    }

    #[inline]
    pub fn core_ptr(&self) -> NonNull<Core> {
        unsafe { NonNull::new_unchecked(self.cur_core) }
    }

    #[inline]
    pub fn get_id(&self) -> ThreadId {
        self.id
    }

    #[inline]
    pub fn set_proc(&mut self, proc: NonNull<Process>) {
        self.proc = proc.as_ptr();
    }

    #[inline]
    pub fn get_proc(&self) -> NonNull<Process> {
        unsafe { NonNull::new_unchecked(self.proc) }
    }

    #[inline]
    pub fn is_kernel_thread(&self) -> bool {
        self.kernel_thread
    }
}

#[derive(Debug)]
#[repr(C, align(8))]
pub struct ArchThread {
    inner: UnsafeCell<ArchThreadInner>,
    lock: Lock,
}

impl ArchThread {
    unsafe fn new(
        proc: NonNull<Process>,
        id: ThreadId,
        kernel_thread: bool,
    ) -> heap::Result<NonNull<Self>> {
        let ptr =
            heap::alloc_layout(Layout::new::<Locked<ArchThread>>()).as_ptr() as *mut ArchThread;
        ptr.write(ArchThread {
            inner: UnsafeCell::new(ArchThreadInner {
                self_ptr: ptr as u64,
                id,
                proc: proc.as_ptr(),
                stack: null_mut(),
                stackframe: Box::new(StackFrame::zeroed()),
                cur_core: null_mut(),
                kernel_thread,
            }),
            lock: Lock::new(),
        });
        Ok(NonNull::new_unchecked(ptr))
    }

    pub fn new_kernel(proc: NonNull<Process>, id: ThreadId) -> heap::Result<NonNull<Self>> {
        unsafe { Self::new(proc, id, true) }
    }

    pub fn new_userspace(proc: NonNull<Process>, id: ThreadId) -> heap::Result<NonNull<Self>> {
        unsafe { Self::new(proc, id, false) }
    }

    #[inline]
    pub fn cur_thread() -> NonNull<Self> {
        let ptr: u64;
        unsafe { asm!("mov rax, gs:[0]", out("rax") ptr) }
        let ptr = ptr as *mut ArchThread;
        unsafe { NonNull::new_unchecked(ptr) }
    }

    pub fn lock(&self) -> LockGuard<ArchThreadInner> {
        self.lock.lock(&self.inner)
    }

    pub unsafe fn get(&self) -> &ArchThreadInner {
        &*self.inner.get()
    }

    pub unsafe fn get_mut(&mut self) -> &mut ArchThreadInner {
        self.inner.get_mut()
    }

    pub fn as_ptr(&self) -> *const ArchThread {
        self as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut ArchThread {
        self as *mut _
    }
}

pub fn free(ptr: *mut ArchThread) {
    heap::free(ptr)
}

pub fn get_gs_base() -> VirtAdr {
    VirtAdr::new(msr::get(msr::GS_BASE))
}

pub fn get_kernel_gs_base() -> VirtAdr {
    VirtAdr::new(msr::get(msr::KERNEL_GS_BASE))
}

unsafe fn set_kernel_gs_base(ptr: *mut ArchThread) {
    msr::set(msr::KERNEL_GS_BASE, ptr as u64);
}

unsafe fn set_gs_base(ptr: *mut ArchThread) {
    msr::set(msr::GS_BASE, ptr as u64);
}

#[inline]
pub fn cur_thread() -> NonNull<ArchThread> {
    ArchThread::cur_thread()
}

pub unsafe fn set_thread(thread: &mut ArchThread) {
    let core_ptr = super::cpu::get_core();
    debug_assert!(!core_ptr.is_null());
    thread.get_mut().cur_core = core_ptr;
    set_gs_base(thread.as_mut_ptr());
    set_kernel_gs_base(null_mut());
}
