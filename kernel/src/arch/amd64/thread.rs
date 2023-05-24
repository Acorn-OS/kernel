use super::msr;
use crate::arch::interrupt::StackFrame;
use crate::mm::heap;
use crate::process::Process;
use crate::util::adr::VirtAdr;
use core::alloc::Layout;
use core::arch::asm;
use core::ptr::NonNull;

pub struct ThreadId(u64);

impl ThreadId {
    pub fn new(unique_id: u64) -> Self {
        Self(unique_id)
    }
}

#[repr(C, align(8))]
pub struct Thread {
    self_ptr: u64,
    pub(super) id: ThreadId,
    pub(super) proc: NonNull<Process>,
    pub(super) stackframe: StackFrame,
    pub(super) stack: *mut u8,
}

impl Thread {
    #[inline]
    pub unsafe fn update_stackframe(&mut self, stackframe: StackFrame) {
        self.stackframe = stackframe
    }

    #[inline]
    pub fn get_stackframe(&self) -> StackFrame {
        self.stackframe.clone()
    }
}

pub unsafe fn new(
    process: NonNull<Process>,
    id: ThreadId,
    entry: VirtAdr,
    stack: VirtAdr,
) -> NonNull<Thread> {
    let entry = entry.adr();
    let ptr = heap::alloc_layout(Layout::new::<Thread>()).as_ptr() as *mut Thread;
    ptr.write(Thread {
        self_ptr: ptr as u64,
        id,
        proc: process,
        stack: stack.ptr() as *mut _,
        stackframe: StackFrame::new_userspace(
            entry,
            stack.adr(),
            process.as_ref().vmm.as_ref().get_page_map(),
        ),
    });
    NonNull::new_unchecked(ptr)
}

pub fn free(ptr: *mut Thread) {
    heap::free(ptr)
}

#[inline]
pub fn cur_thread() -> NonNull<Thread> {
    let ptr: u64;
    unsafe { asm!("mov rax, fs:[0]", out("rax") ptr) }
    let ptr = ptr as *mut Thread;
    debug_assert!(!ptr.is_null());
    unsafe { NonNull::new_unchecked(ptr) }
}

pub unsafe fn set_thread(thread: NonNull<Thread>) {
    msr::set(msr::FS_BASE, thread.addr().get() as u64);
}
