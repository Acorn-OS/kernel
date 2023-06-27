macro_rules! def_locked_ptr {
    ($ident:ident, $ty:ty) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $ident(*const UnsafeCell<$ty>);

        impl $ident {
            pub const unsafe fn nullptr() -> Self {
                Self(null_mut())
            }

            pub const unsafe fn from_ptr(ptr: *mut $ty) -> Self {
                Self(ptr as *const _)
            }

            pub fn get_locked<'a>(self) -> LockGuard<'a, $ty> {
                unsafe { (*(*self.0).get()).lock.lock(&*self.0) }
            }

            pub unsafe fn get<'a>(self) -> &'a $ty {
                &*(*self.0).get()
            }

            pub unsafe fn get_mut<'a>(self) -> &'a mut $ty {
                &mut *(*self.0).get()
            }

            pub fn is_null(self) -> bool {
                self.0.is_null()
            }

            pub fn as_ptr(self) -> *const $ty {
                self.0 as *const _
            }

            pub fn as_adr(self) -> VirtAdr {
                VirtAdr::new(self.0 as u64)
            }
        }
    };
}

pub mod loader;
pub mod thread;

mod error;

use self::thread::ThreadPtr;
use crate::mm::heap;
use crate::mm::vmm::VMM;
use crate::util::adr::VirtAdr;
use crate::util::locked::{LockGuard, LockPrimitive};
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::fmt::Display;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicU16, Ordering};

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

#[derive(Debug)]
#[repr(C)]
pub struct Process {
    lock: LockPrimitive,
    pub vmm: VMM,
    pub id: ProcessId,
    thread_id_counter: u64,
    pub threads: Vec<ThreadPtr>,
}

impl Process {
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

    pub unsafe fn lock(&self) {
        self.lock.manually_lock();
    }

    pub unsafe fn unlock(&self) {
        self.lock.manually_unlock();
    }

    fn as_ptr(&self) -> *const Self {
        self as *const Self
    }

    fn as_mut_ptr(&mut self) -> *mut Self {
        self as *mut Self
    }

    fn add_thread(&mut self, thread: ThreadPtr) -> Result<()> {
        self.threads.push(thread);
        Ok(())
    }

    fn remove_thread(&mut self, _thread_id: ThreadPtr) -> Result<()> {
        Ok(())
    }
}

def_locked_ptr!(ProcessPtr, Process);

pub fn get(id: ProcessId) -> Option<ProcessPtr> {
    let ptr = unsafe { PROCESSES[id.0 as usize] };
    if !ptr.is_null() {
        Some(unsafe { ProcessPtr::from_ptr(ptr) })
    } else {
        None
    }
}

pub fn new_proc(vmm: VMM) -> Result<(ProcessPtr, ProcessId)> {
    let index = PROCESS_COUNTER.fetch_add(1, Ordering::Relaxed);
    let id = ProcessId(index);
    info!("creating new process with ID: {index}");
    let process = unsafe {
        PROCESSES[index as usize] = heap::alloc(Process {
            lock: LockPrimitive::new(),
            vmm,
            id,
            thread_id_counter: 0,
            threads: vec![],
        })
        .as_ptr();
        PROCESSES[index as usize]
    };
    Ok((unsafe { ProcessPtr::from_ptr(process) }, id))
}
