use crate::arch::interrupt::StackFrame;

pub struct Thread {
    pub(super) kernel_stackframe: *mut StackFrame,
    pub(super) stack: *mut u8,
}

impl Thread {
    pub unsafe fn new(stack: *mut u8, entry: u64) -> Self {
        let entry = entry as u64;
        (stack.sub(core::mem::size_of::<StackFrame>()) as *mut StackFrame)
            .write(StackFrame::new_kernel(entry, stack as u64));
        Self {
            kernel_stackframe: stack.sub(core::mem::size_of::<StackFrame>()) as *mut _,
            stack,
        }
    }
}
