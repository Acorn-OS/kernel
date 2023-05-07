use crate::arch::interrupt::StackFrame;
use crate::arch::vm;
use crate::mm::pmm;
use crate::mm::vmm::{Flags, VirtualMemory, PAGE_SIZE};

pub struct Thread {
    pub(super) kernel_stackframe: *mut StackFrame,
    pub(super) stack: *mut u8,
}

impl Thread {
    pub fn new(vmm: &mut VirtualMemory, entry: u64, stack_pages: usize) -> Self {
        let entry = entry as u64;
        unsafe {
            let total_bytes = stack_pages * PAGE_SIZE;
            let stackframe_size = core::mem::size_of::<StackFrame>();
            let alloc = pmm::alloc_pages(stack_pages);
            let virt_stack_top = alloc.as_virt_ptr::<u8>().add(total_bytes);
            let stack = vmm.map(
                None,
                stack_pages,
                Flags::Phys {
                    flags: vm::Flags::PRESENT | vm::Flags::RW,
                    phys: alloc.phys_adr(),
                },
            ) + total_bytes as u64;
            (virt_stack_top.sub(stackframe_size) as *mut StackFrame)
                .write(StackFrame::new_kernel(entry, stack as u64));
            let stack_frame_ptr = stack - stackframe_size as u64;
            Self::from_raw(stack as *mut _, stack_frame_ptr as *mut _)
        }
    }

    pub unsafe fn from_raw(stack: *mut u8, stackframe: *mut StackFrame) -> Self {
        Self {
            kernel_stackframe: stackframe,
            stack,
        }
    }
}
