#[repr(C)]
pub struct StackFrame {
    rbp: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64,
    kind: u64,
    error: u64,
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

macro_rules! irq_handler {
    ($($fn:ident),*) => {
        $(
            pub fn $fn();
        )*
    };
}

extern "sysv64" {
    irq_handler! {
        _irq_handler_0,
        _irq_handler_1,
        _irq_handler_2,
        _irq_handler_3,
        _irq_handler_4,
        _irq_handler_5,
        _irq_handler_6,
        _irq_handler_7,
        _irq_handler_8,
        _irq_handler_9,
        _irq_handler_10,
        _irq_handler_11,
        _irq_handler_12,
        _irq_handler_13,
        _irq_handler_14,
        _irq_handler_15,
        _irq_handler_16,
        _irq_handler_17,
        _irq_handler_18,
        _irq_handler_19,
        _irq_handler_20,
        _irq_handler_21,
        _irq_handler_22,
        _irq_handler_23,
        _irq_handler_24,
        _irq_handler_25,
        _irq_handler_26,
        _irq_handler_27,
        _irq_handler_28,
        _irq_handler_29,
        _irq_handler_30,
        _irq_handler_31
    }
}

#[no_mangle]
pub unsafe extern "sysv64" fn unimp() {
    panic!("unimplemented handler")
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_division_error(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_debug(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_non_maskable_interrupt(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_breakpoint(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_overflow(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_bound_range_exceeded(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_invalid_opcode(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_device_not_available(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_double_fault(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_deprecated(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_invalid_tss(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_segment_not_present(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_stack_segment_fault(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_general_protection_fault(
    _stackframe: *mut StackFrame,
) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_page_fault(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_reserved(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_x87_floating_point(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_alignment_check(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_machine_check(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_simd_floating_point(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_virtualization(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_control_protection(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_hypervisor_injection(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_vmm_communication(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "sysv64" fn excpt_security(_stackframe: *mut StackFrame) -> StackFrame {
    unimplemented!()
}
