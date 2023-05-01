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
                kernel_empty_proc as u64,
            ),
        )
    };
    id
}

extern "C" fn kernel_empty_proc() -> ! {
    loop {
        info!("empty proc");
        interrupt::halt();
    }
}
