use crate::arch::interrupt::StackFrame;
use crate::arch::thread::{ArchThread, ArchThreadInner};
use crate::arch::{self, vm};
use crate::mm::vmm::{Flags, VirtualMemory, PAGE_SIZE};
use crate::mm::{heap, pmm};
use crate::process::Process;
use crate::util::adr::VirtAdr;
use crate::util::locked::LockGuard;
use core::fmt::{self, Display};
use core::ptr::NonNull;

pub unsafe fn create_userspace_thread_stack(vmm: &mut VirtualMemory, pages: usize) -> VirtAdr {
    let total_bytes = pages * PAGE_SIZE;
    let alloc = pmm::alloc_pages(pages);
    let virt_adr = VirtAdr::new((((1 << 47) - PAGE_SIZE * 2) - PAGE_SIZE * 512) as u64);
    vmm.map(
        Some(virt_adr),
        pages,
        Flags::Phys {
            flags: vm::Flags::PRESENT | vm::Flags::RW | vm::Flags::USER | vm::Flags::XD,
            phys: alloc.phys(),
        },
    )
    .add(total_bytes)
}

#[derive(Debug, Clone, Copy)]
pub struct ThreadId(usize);

impl Display for ThreadId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Thread {
    inner: ArchThread,
}

impl Thread {
    unsafe fn wrap_ptr(ptr: NonNull<ArchThread>) -> NonNull<Thread> {
        let ptr = ptr.as_ptr();
        NonNull::new_unchecked(ptr as *mut Thread)
    }

    #[inline]
    pub fn cur_thread() -> NonNull<Self> {
        unsafe { Self::wrap_ptr(ArchThread::cur_thread()) }
    }

    pub fn lock(&self) -> LockGuard<ArchThreadInner> {
        self.inner.lock()
    }

    pub unsafe fn get(&self) -> &ArchThreadInner {
        self.inner.get()
    }

    pub unsafe fn get_mut(&mut self) -> &mut ArchThreadInner {
        self.inner.get_mut()
    }

    pub fn as_ptr(&self) -> *const ArchThread {
        self.inner.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut ArchThread {
        self.inner.as_mut_ptr()
    }
}

unsafe fn new(
    mut thread: NonNull<ArchThread>,
    stackframe: StackFrame,
) -> heap::Result<NonNull<Thread>> {
    let mref = thread.as_mut();
    let mref = mref.get_mut();
    mref.set_stackframe(stackframe);
    Ok(Thread::wrap_ptr(thread))
}

pub unsafe fn new_kernel(
    proc: NonNull<Process>,
    entry: VirtAdr,
    stack: VirtAdr,
) -> heap::Result<NonNull<Thread>> {
    let thread = ArchThread::new_kernel(proc, ThreadId(0))?;
    new(
        thread,
        StackFrame::new_kernel(
            entry.adr(),
            stack.adr(),
            proc.as_ref().get().vmm.get_page_map(),
        ),
    )
}

pub unsafe fn new_userspace(
    proc: NonNull<Process>,
    entry: VirtAdr,
    stack: VirtAdr,
) -> heap::Result<NonNull<Thread>> {
    let thread = ArchThread::new_userspace(proc, ThreadId(0))?;
    new(
        thread,
        StackFrame::new_userspace(
            entry.adr(),
            stack.adr(),
            proc.as_ref().get().vmm.get_page_map(),
        ),
    )
}

pub fn free(_thread: NonNull<Thread>) {}

pub fn cur_thread() -> NonNull<Thread> {
    unsafe { Thread::wrap_ptr(arch::thread::cur_thread()) }
}

pub fn set_thread(mut thread: NonNull<Thread>) {
    unsafe { arch::thread::set_thread(&mut thread.as_mut().inner) }
}
