use crate::arch::interrupt;
use crate::mm::{heap, pmm, vmm};
use crate::process::thread::Thread;
use crate::process::{new_kernel_proc, ProcessId};

pub fn new() -> ProcessId {
    let vmm = vmm::new_kernel();
    let (_, id) = unsafe {
        new_kernel_proc(
            heap::alloc(vmm),
            Thread::new(
                (pmm::alloc_pages(256).virt_adr() + pmm::PAGE_SIZE as u64 * 256) as *mut _,
                kernel_process_main as u64,
            ),
        )
    };
    id
}

extern "C" fn kernel_process_main() -> ! {
    info!("hello from kernel process main!");
    loop {
        info!("kernel proc print!");
        interrupt::halt();
    }
}
