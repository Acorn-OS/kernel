use crate::arch::interrupt::StackFrame;
use crate::arch::vm;
use crate::mm::pmm;
use crate::mm::vmm::{Flags, VirtualMemory, PAGE_SIZE};
use crate::util::adr::VirtAdr;

pub struct Thread {
    pub(super) stackframe: StackFrame,
    pub(super) stack: *mut u8,
}

impl Thread {
    pub fn new(vmm: &mut VirtualMemory, entry: u64, stack_pages: usize) -> Self {
        let entry = entry as u64;
        unsafe {
            let total_bytes = stack_pages * PAGE_SIZE;
            let alloc = pmm::alloc_pages(stack_pages);
            let virt_adr = VirtAdr::new((((1 << 47) - PAGE_SIZE * 2) - PAGE_SIZE * 512) as u64);
            let stack = vmm
                .map(
                    Some(virt_adr),
                    stack_pages,
                    Flags::Phys {
                        flags: vm::Flags::PRESENT | vm::Flags::RW | vm::Flags::USER | vm::Flags::XD,
                        phys: alloc.phys(),
                    },
                )
                .add(total_bytes);
            Self::from_raw(
                stack.ptr() as *mut _,
                StackFrame::new_userspace(entry, stack.adr(), vmm.get_page_map()),
            )
        }
    }

    pub unsafe fn from_raw(stack: *mut u8, stackframe: StackFrame) -> Self {
        Self { stackframe, stack }
    }
}
