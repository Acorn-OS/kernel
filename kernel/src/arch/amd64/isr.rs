use crate::arch::amd64::vm;
use core::arch::global_asm;

global_asm!(include_str!("isr.s"));

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

extern "C" {
    irq_handler! {
        irq_handler_0,
        irq_handler_1,
        irq_handler_2,
        irq_handler_3,
        irq_handler_4,
        irq_handler_5,
        irq_handler_6,
        irq_handler_7,
        irq_handler_8,
        irq_handler_9,
        irq_handler_10,
        irq_handler_11,
        irq_handler_12,
        irq_handler_13,
        irq_handler_14,
        irq_handler_15,
        irq_handler_16,
        irq_handler_17,
        irq_handler_18,
        irq_handler_19,
        irq_handler_20,
        irq_handler_21,
        irq_handler_22,
        irq_handler_23,
        irq_handler_24,
        irq_handler_25,
        irq_handler_26,
        irq_handler_27,
        irq_handler_28,
        irq_handler_29,
        irq_handler_30,
        irq_handler_31,
        irq_handler_32,
        irq_handler_33,
        irq_handler_34,
        irq_handler_35,
        irq_handler_36,
        irq_handler_37,
        irq_handler_38,
        irq_handler_39,
        irq_handler_40,
        irq_handler_41,
        irq_handler_42,
        irq_handler_43,
        irq_handler_44,
        irq_handler_45,
        irq_handler_46,
        irq_handler_47
    }
}

#[no_mangle]
pub unsafe extern "C" fn unimp() -> ! {
    panic!("unimplemented handler");
}

mod excpt {
    use crate::mm::pmm;

    use super::*;
    use core::arch::asm;

    #[no_mangle]
    unsafe extern "C" fn excpt_division_error(_stackframe: *mut StackFrame) -> *mut StackFrame {
        unimplemented!();
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_debug(_stackframe: *mut StackFrame) -> *mut StackFrame {
        unimplemented!();
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_non_maskable_interrupt(
        _stackframe: *mut StackFrame,
    ) -> *mut StackFrame {
        unimplemented!();
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_breakpoint(_stackframe: *mut StackFrame) -> *mut StackFrame {
        unimplemented!();
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_overflow(_stackframe: *mut StackFrame) -> *mut StackFrame {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_bound_range_exceeded(
        _stackframe: *mut StackFrame,
    ) -> *mut StackFrame {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_invalid_opcode(_stackframe: *mut StackFrame) -> *mut StackFrame {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_device_not_available(
        _stackframe: *mut StackFrame,
    ) -> *mut StackFrame {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_double_fault(_stackframe: *mut StackFrame) -> *mut StackFrame {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_deprecated(_stackframe: *mut StackFrame) -> *mut StackFrame {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_invalid_tss(_stackframe: *mut StackFrame) -> *mut StackFrame {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_segment_not_present(
        _stackframe: *mut StackFrame,
    ) -> *mut StackFrame {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_stack_segment_fault(
        _stackframe: *mut StackFrame,
    ) -> *mut StackFrame {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_general_protection_fault(
        _stackframe: *mut StackFrame,
    ) -> *mut StackFrame {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_page_fault(stackframe: *mut StackFrame) -> *mut StackFrame {
        let adr: u64;
        asm!(
            "mov {adr}, cr2",
            adr = out(reg) adr
        );
        if adr == 0 {
            panic!("nullptr")
        }
        if let Some(entry) = vm::get_page_entry(vm::get_cur(), adr) {
            if (*entry).is_resv() {
                (*entry).set_adr(pmm::alloc_pages_zeroed(1) as u64);
                (*entry).set_present()
            } else {
                panic!("accessed an unreserved page");
            }
        } else {
            panic!("invalid memory access!")
        }
        stackframe
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_reserved(_stackframe: *mut StackFrame) -> *mut StackFrame {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_x87_floating_point(_stackframe: *mut StackFrame) -> *mut StackFrame {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_alignment_check(_stackframe: *mut StackFrame) -> *mut StackFrame {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_machine_check(_stackframe: *mut StackFrame) -> *mut StackFrame {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_simd_floating_point(
        _stackframe: *mut StackFrame,
    ) -> *mut StackFrame {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_virtualization(_stackframe: *mut StackFrame) -> *mut StackFrame {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_control_protection(_stackframe: *mut StackFrame) -> *mut StackFrame {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_hypervisor_injection(
        _stackframe: *mut StackFrame,
    ) -> *mut StackFrame {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_vmm_communication(_stackframe: *mut StackFrame) -> *mut StackFrame {
        unimplemented!()
    }

    #[no_mangle]
    unsafe extern "C" fn excpt_security(_stackframe: *mut StackFrame) -> *mut StackFrame {
        unimplemented!()
    }
}

mod irq {
    use crate::arch::amd64::lapic;

    use super::*;

    #[no_mangle]
    unsafe extern "C" fn irq_timer(stackframe: *mut StackFrame) -> *mut StackFrame {
        info!("timer!");
        lapic::eoi();
        stackframe
    }
}
