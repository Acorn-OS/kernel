pub mod loader;
pub mod scheduler;
pub mod thread;

use self::thread::Thread;
use crate::arch::interrupt;
use crate::fs::initrd::InitrdFs;
use crate::fs::Vfs;
use crate::mm::heap;
use crate::mm::vmm::VirtualMemory;
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

pub struct Process {
    vmm: NonNull<VirtualMemory>,
    id: ProcessId,
    main_thread: Thread,
}

pub fn get(id: ProcessId) -> Option<NonNull<Process>> {
    let ptr = unsafe { PROCESSES[id.0 as usize] };
    if !ptr.is_null() {
        Some(unsafe { NonNull::new_unchecked(ptr) })
    } else {
        None
    }
}

pub fn new_kernel_proc(
    vmm: NonNull<VirtualMemory>,
    main_thread: Thread,
) -> (NonNull<Process>, ProcessId) {
    let index = PROCESS_COUNTER.fetch_add(1, Ordering::Relaxed);
    let id = ProcessId(index);
    info!("new kernel proc: '{:?}'", vmm.as_ptr());
    let process = unsafe {
        PROCESSES[index as usize] = heap::alloc(Process {
            vmm,
            id,
            main_thread,
        })
        .as_ptr();
        PROCESSES[index as usize]
    };
    (unsafe { NonNull::new_unchecked(process) }, id)
}

pub fn run(mut initrd: InitrdFs) -> ! {
    for file in initrd.ls("").expect("failed to list files in initrd") {
        info!("starting module '{}'", file.name);
        let open = initrd
            .open(&file.name)
            .expect(&format!("failed to open file '{}' from initrd", file.name));
        let elf = elf::Elf64::parse_elf(
            initrd
                .read(open)
                .expect(&format!("failed to read file '{}' from initrd", file.name))
                .into_boxed_slice(),
        )
        .expect(&format!(
            "unable to parse file '{}' as executable elf",
            file.name
        ));
        let (_, id) = loader::elf::spawn(&elf);
        scheduler::schedule(id);
    }
    interrupt::enable();
    loop {
        interrupt::halt();
    }
}
