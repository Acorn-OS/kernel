pub mod sched;

use crate::arch;
use crate::arch::interrupt::StackFrame;
use crate::arch::thread::ArchThread;
use crate::mm::vmm::{Flags, MapTy, PAGE_SIZE, VMM};
use crate::mm::{heap, pmm};
use crate::util::adr::VirtAdr;
use crate::util::locked::{LockGuard, LockPrimitive};
use alloc::boxed::Box;
use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::fmt::{self, Debug, Display};
use core::ptr::{null_mut, NonNull};

use super::ProcessPtr;

pub unsafe fn create_userspace_thread_stack(vmm: &mut VMM, pages: usize) -> VirtAdr {
    let total_bytes = pages * PAGE_SIZE;
    let alloc = pmm::alloc_pages(pages);
    let virt_adr = VirtAdr::new((((1 << 47) - PAGE_SIZE * 2) - PAGE_SIZE * 512) as u64);
    vmm.map(
        Some(virt_adr),
        pages,
        Flags::RW | Flags::USER,
        MapTy::Phys { adr: alloc.phys() },
    )
    .unwrap()
    .add(total_bytes)
}

#[derive(Debug, Clone, Copy)]
pub struct ThreadId(usize);

impl ThreadId {
    fn from_index(index: usize) -> Self {
        Self(index)
    }

    pub fn gen() -> Self {
        Self(0)
    }

    pub fn resv_id(id: usize) -> Self {
        Self::from_index(id)
    }
}

impl Display for ThreadId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ThreadStatus {
    Waiting,
    Sleeping,
    Running,
}

#[derive(Debug, Clone, Copy)]
pub enum ThreadScheduleStatus {
    Sleep,
    Running,
}

/// Most methods require a mutable access, which can be acquired by the `lock` function.
/// The `lock` function locks the thread, allowing exclusive mutable access to that current thread.
#[derive(Debug)]
#[repr(C)]
pub struct Thread {
    arch: ArchThread,
    lock: LockPrimitive,
    id: ThreadId,
    proc: ProcessPtr,
    stackframe: Box<StackFrame>,
    stack: *mut u8,
    status: ThreadStatus,
    schedule_status: ThreadScheduleStatus,
}

impl Thread {
    pub unsafe fn new(
        id: ThreadId,
        proc: ProcessPtr,
        stackframe: Box<StackFrame>,
    ) -> heap::Result<ThreadPtr> {
        let ptr = heap::alloc_layout(Layout::new::<Thread>()).as_ptr() as *mut Thread;
        ptr.write(Thread {
            id,
            lock: LockPrimitive::new(),
            arch: ArchThread::new(),
            proc,
            stack: null_mut(),
            stackframe,
            status: ThreadStatus::Waiting,
            schedule_status: ThreadScheduleStatus::Running,
        });
        ArchThread::init(&mut *ptr);
        Ok(ThreadPtr::from_ptr(ptr))
    }

    #[inline]
    pub unsafe fn cur_thread() -> ThreadPtr {
        arch::thread::cur_thread()
    }

    #[inline]
    pub fn make_thread_current(thread: &mut Thread) {
        unsafe {
            thread.proc.get().vmm.install();
            arch::thread::set_thread(thread)
        }
    }

    pub fn as_ptr(&self) -> ThreadPtr {
        // works because Thread uses unsafe Cell
        ThreadPtr(self as *const _ as *const _)
    }

    #[inline]
    pub fn get_id(&self) -> ThreadId {
        self.id
    }

    #[inline]
    pub fn arch_mut(&mut self) -> &mut ArchThread {
        &mut self.arch
    }

    #[inline]
    pub fn arch(&self) -> &ArchThread {
        &self.arch
    }

    #[inline]
    pub unsafe fn set_stackframe(&mut self, stackframe: StackFrame) {
        *self.stackframe = stackframe
    }

    #[inline]
    pub fn get_stackframe(&self) -> &StackFrame {
        &self.stackframe
    }

    #[inline]
    pub fn set_proc(&mut self, proc: ProcessPtr) {
        self.proc = proc;
    }

    #[inline]
    pub fn get_proc(&self) -> ProcessPtr {
        self.proc
    }

    #[inline]
    pub fn get_status(&self) -> ThreadStatus {
        self.status
    }

    /// # Safety
    /// if set to an incorrect status it could lead to incorrect behaviour.
    #[inline]
    pub fn set_status(&mut self, status: ThreadStatus) {
        self.status = status;
    }

    #[inline]
    pub fn get_schedule_status(&self) -> ThreadScheduleStatus {
        self.schedule_status
    }

    #[inline]
    pub fn set_schedule_status(&mut self, status: ThreadScheduleStatus) {
        self.schedule_status = status
    }
}

def_locked_ptr!(ThreadPtr, Thread);

pub unsafe fn new(
    id: ThreadId,
    proc: ProcessPtr,
    stackframe: Box<StackFrame>,
) -> heap::Result<ThreadPtr> {
    Thread::new(id, proc, stackframe)
}

pub fn free(_thread: NonNull<Thread>) {}

pub unsafe fn cur_thread() -> ThreadPtr {
    let thread = Thread::cur_thread();
    debug_assert!(!thread.is_null());
    thread
}

pub unsafe fn make_thread_current(thread: &mut Thread) {
    Thread::make_thread_current(thread)
}
