use super::process;
use crate::arch::interrupt;
use crate::arch::vm;
use crate::boot::BootInfo;
use crate::mm::pmm;
use crate::mm::vmm;
use crate::process::thread;
use crate::process::thread::Thread;
use crate::process::Process;
use crate::scheduler;
use crate::util::adr::VirtAdr;
use core::ptr::null_mut;

static mut INIT_PROC: *mut Process = null_mut();
static mut EMPTY_THREAD: *mut Thread = null_mut();

pub unsafe fn run(boot_info: BootInfo) -> ! {
    trace!("initializing init processes");
    let mut vmm = vmm::create_kernel_vmm(&boot_info);
    let stack = vmm
        .map(
            None,
            256,
            vmm::Flags::Phys {
                flags: vm::Flags::PRESENT | vm::Flags::RW | vm::Flags::XD,
                phys: pmm::alloc_pages(256).phys(),
            },
        )
        .add(256 * vmm::PAGE_SIZE);
    let proc = process::new_proc(vmm).unwrap().0;
    INIT_PROC = proc.as_ptr();
    let empty_thread = thread::new_kernel(proc, VirtAdr::new(0), VirtAdr::new(0)).expect("");
    EMPTY_THREAD = empty_thread.as_ptr();
    let thread = thread::new_kernel(proc, VirtAdr::new(main as u64), stack)
        .expect("failed to create thread");
    thread::set_thread(empty_thread);
    scheduler::schedule(thread);
    trace!("starting kernel main thread.");
    interrupt::enable();
    loop {
        interrupt::halt();
    }
}

fn main() -> ! {
    info!("entered kernel main...");
    //info!("loaded modules count: {}", boot_info.modules.module_count);
    //info!("loaded modules:");
    //for i in 0..boot_info.modules.module_count as usize {
    //    let path = unsafe {
    //        let ptr = boot_info.modules.modules.as_ptr().add(i);
    //        CStr::from_ptr((*ptr).path.as_ptr().unwrap())
    //    };
    //    info!("    {}", String::from_utf8_lossy(path.to_bytes()));
    //}
    loop {
        interrupt::halt();
    }
}
