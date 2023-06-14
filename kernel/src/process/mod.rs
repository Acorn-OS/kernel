pub mod loader;
pub mod thread;

mod error;

use crate::mm::heap;
use crate::mm::vmm::VirtualMemory;
use crate::util::locked::{Lock, LockGuard};
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::fmt::Display;
use core::ptr::{null_mut, NonNull};
use core::sync::atomic::{AtomicU16, Ordering};
use thread::Thread;

pub use error::{Error, Result};

type ProcessIdPrimitive = u16;
#[derive(Clone, Copy, Debug)]
pub struct ProcessId(ProcessIdPrimitive);

impl Display for ProcessId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

const MAX_PROCESSES: usize = ProcessIdPrimitive::MAX as usize;

static mut PROCESSES: [*mut Process; MAX_PROCESSES] = [null_mut(); MAX_PROCESSES];
static PROCESS_COUNTER: AtomicU16 = AtomicU16::new(0);

#[repr(C)]
pub struct ProcessInner {
    pub vmm: VirtualMemory,
    pub id: ProcessId,
    thread_id_counter: u64,
    pub threads: Vec<NonNull<Thread>>,
}

impl ProcessInner {
    //pub fn add_thread(&mut self, entry: VirtAdr, stack_pages: usize) -> NonNull<Thread> {
    //    let stack = unsafe { create_thread_stack(self.vmm, stack_pages) };
    //    let id = self.gen_new_thread_id();
    //    let thread = unsafe { thread::new_userspace(self.as_ptr(), id, entry, stack) };
    //    self.append_thread_raw(thread);
    //    thread
    //}
    //
    //pub fn add_kernel_thread(&mut self, entry: VirtAdr, stack_pages: usize) -> NonNull<Thread> {
    //    let stack = unsafe { create_thread_stack(self.vmm, stack_pages) };
    //    let id = self.gen_new_thread_id();
    //    let thread = unsafe { thread::new_kernel(self.as_ptr(), id, entry, stack) };
    //    self.append_thread_raw(thread);
    //    thread
    //}

    fn as_ptr(&self) -> *const Self {
        self as *const Self
    }

    fn as_mut_ptr(&mut self) -> *mut Self {
        self as *mut Self
    }

    fn add_thread(&mut self, thread: NonNull<Thread>) -> Result<()> {
        self.threads.push(thread);
        Ok(())
    }

    fn remove_thread(&mut self, _thread_id: NonNull<Thread>) -> Result<()> {
        Ok(())
    }
}

#[repr(C)]
pub struct Process {
    inner: UnsafeCell<ProcessInner>,
    lock: Lock,
}

impl Process {
    pub fn lock(&self) -> LockGuard<ProcessInner> {
        self.lock.lock(&self.inner)
    }

    pub unsafe fn get(&self) -> &ProcessInner {
        &*self.inner.get()
    }

    pub unsafe fn get_mut(&mut self) -> &mut ProcessInner {
        self.inner.get_mut()
    }
}

pub fn get(id: ProcessId) -> Option<NonNull<Process>> {
    let ptr = unsafe { PROCESSES[id.0 as usize] };
    if !ptr.is_null() {
        Some(unsafe { NonNull::new_unchecked(ptr) })
    } else {
        None
    }
}

pub fn new_proc(vmm: VirtualMemory) -> Result<(NonNull<Process>, ProcessId)> {
    let index = PROCESS_COUNTER.fetch_add(1, Ordering::Relaxed);
    let id = ProcessId(index);
    info!("creating new process with ID: {index}");
    let process = unsafe {
        PROCESSES[index as usize] = heap::alloc(Process {
            lock: Lock::new(),
            inner: UnsafeCell::new(ProcessInner {
                vmm,
                id,
                thread_id_counter: 0,
                threads: vec![],
            }),
        })
        .as_ptr();
        PROCESSES[index as usize]
    };
    Ok((unsafe { NonNull::new_unchecked(process) }, id))
}
