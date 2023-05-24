use super::Process;
use crate::arch::{interrupt, thread};
use crate::mm::{heap, vmm};
use crate::util::adr::VirtAdr;
use core::ptr::null_mut;

static mut INIT_PROC: *mut Process = null_mut();

pub unsafe fn init() -> ! {
    trace!("initializing init processes");
    let vmm = heap::alloc(vmm::new_userland());
    INIT_PROC = super::new_proc(vmm).0.as_ptr();
    let thread = (*INIT_PROC).add_thread(VirtAdr::new(0), 256);
    thread::set_thread(thread);
    init_main();
}

fn init_main() -> ! {
    info!("starting processes");
    interrupt::enable();
    loop {
        info!("printing!");
        interrupt::halt();
    }
}
