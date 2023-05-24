pub mod init;
pub mod loader;
pub mod scheduler;

use crate::arch::thread::{Thread, ThreadId};
use crate::arch::{thread, vm};
use crate::mm::pmm::PAGE_SIZE;
use crate::mm::vmm::{Flags, VirtualMemory};
use crate::mm::{heap, pmm};
use crate::util::adr::VirtAdr;
use alloc::vec::Vec;
use core::fmt::Display;
use core::ptr::{null_mut, NonNull};
use core::sync::atomic::{AtomicU16, Ordering};

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

unsafe fn create_thread_stack(mut vmm: NonNull<VirtualMemory>, pages: usize) -> VirtAdr {
    let total_bytes = pages * PAGE_SIZE;
    let alloc = pmm::alloc_pages(pages);
    let virt_adr = VirtAdr::new((((1 << 47) - PAGE_SIZE * 2) - PAGE_SIZE * 512) as u64);
    vmm.as_mut()
        .map(
            Some(virt_adr),
            pages,
            Flags::Phys {
                flags: vm::Flags::PRESENT | vm::Flags::RW | vm::Flags::USER | vm::Flags::XD,
                phys: alloc.phys(),
            },
        )
        .add(total_bytes)
}

pub struct Process {
    pub vmm: NonNull<VirtualMemory>,
    pub id: ProcessId,
    thread_id_counter: u64,
    threads: Vec<NonNull<Thread>>,
}

impl Process {
    pub fn add_thread(&mut self, entry: VirtAdr, stack_pages: usize) -> NonNull<Thread> {
        let stack = unsafe { create_thread_stack(self.vmm, stack_pages) };
        let id = self.gen_new_thread_id();
        let thread = unsafe { thread::new(self.as_ptr(), id, entry, stack) };
        self.append_thread_raw(thread);
        thread
    }

    fn gen_new_thread_id(&mut self) -> ThreadId {
        let id = self.thread_id_counter;
        self.thread_id_counter += 1;
        ThreadId::new(id)
    }

    fn as_ptr(&mut self) -> NonNull<Self> {
        let ptr = self as *mut Self;
        debug_assert!(!ptr.is_null());
        unsafe { NonNull::new_unchecked(ptr) }
    }

    fn append_thread_raw(&mut self, thread: NonNull<Thread>) {
        self.threads.push(thread);
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

pub fn new_proc(vmm: NonNull<VirtualMemory>) -> (NonNull<Process>, ProcessId) {
    let index = PROCESS_COUNTER.fetch_add(1, Ordering::Relaxed);
    let id = ProcessId(index);
    info!("creating new process with ID: {index}");
    let process = unsafe {
        PROCESSES[index as usize] = heap::alloc(Process {
            vmm,
            id,
            thread_id_counter: 0,
            threads: vec![],
        })
        .as_ptr();
        PROCESSES[index as usize]
    };
    (unsafe { NonNull::new_unchecked(process) }, id)
}

pub fn run() -> ! {
    unsafe { init::init() }
}
