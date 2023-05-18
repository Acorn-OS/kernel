use core::arch::asm;

#[repr(C, packed)]
pub struct StackFrame {
    rbp: *const StackFrame,
    rip: u64,
}

impl StackFrame {
    pub unsafe fn from_current_stackframe() -> *const Self {
        let rbp: u64;
        asm!(
            "mov rax, rbp",
            out("rax") rbp
        );
        rbp as *const _
    }

    pub fn ip(&self) -> u64 {
        self.rip
    }

    pub fn next(&self) -> *const StackFrame {
        self.rbp
    }
}
