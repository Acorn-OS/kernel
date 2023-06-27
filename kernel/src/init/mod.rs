mod main;

use super::process;
use crate::arch::interrupt;
use crate::arch::interrupt::StackFrame;
use crate::boot::BootInfo;
use crate::drivers;
use crate::mm::heap;
use crate::mm::pmm;
use crate::mm::vmm;
use crate::mm::vmm::Flags;
use crate::mm::vmm::VMM;
use crate::process::thread::{self, sched, ThreadId, ThreadPtr, ThreadScheduleStatus};
use crate::process::ProcessPtr;
use crate::util::adr::VirtAdr;
use alloc::boxed::Box;

static mut INIT_PROC: ProcessPtr = unsafe { ProcessPtr::nullptr() };
static mut EMPTY_THREAD: ThreadPtr = unsafe { ThreadPtr::nullptr() };

unsafe fn create_init_thread_stack(vmm: &mut VMM) -> VirtAdr {
    vmm.map(
        None,
        256,
        Flags::PRESENT | Flags::RW | Flags::XD,
        pmm::alloc_pages(256).phys(),
    )
    .add(256 * vmm::PAGE_SIZE)
}

unsafe fn create_init_proc_thread(
    id: ThreadId,
    proc: ProcessPtr,
    vmm: &mut VMM,
    entry: unsafe extern "C" fn() -> !,
) -> ThreadPtr {
    trace!("creating init thread {id}");
    let entry = entry as u64;
    let stack = create_init_thread_stack(vmm);
    let stackframe = Box::new(StackFrame::new_kernel(
        entry,
        stack.adr(),
        vmm.get_page_map(),
    ));
    let thread = thread::new(id, proc, stackframe).expect("failed to create init proc thread");
    sched::schedule(thread);
    thread
}

pub fn init_proc() -> ProcessPtr {
    unsafe {
        debug_assert!(!INIT_PROC.is_null());
        INIT_PROC
    }
}

pub unsafe fn run(boot_info: BootInfo) -> ! {
    trace!("initializing init processes");

    let vmm = vmm::create_kernel(&boot_info);
    vmm.install();
    let proc = process::new_proc(vmm).unwrap().0;
    INIT_PROC = proc;

    trace!("initializing kernel heap");
    heap::init();

    let empty_thread =
        thread::new(ThreadId::resv_id(0), proc, Box::new(StackFrame::zeroed())).unwrap();
    empty_thread
        .get_mut()
        .set_schedule_status(ThreadScheduleStatus::Sleep);
    thread::make_thread_current(empty_thread.get_mut());
    EMPTY_THREAD = empty_thread;

    let vmm = &mut proc.get_mut().vmm;
    create_init_proc_thread(ThreadId::resv_id(1), proc, vmm, main::main);
    create_init_proc_thread(ThreadId::resv_id(2), proc, vmm, drivers::kbd::main);
    create_init_proc_thread(ThreadId::resv_id(3), proc, vmm, drivers::crsr::main);

    trace!("starting kernel main thread.");
    interrupt::enable();
    loop {
        interrupt::halt();
    }
}
