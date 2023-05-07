use super::super::vm;
use super::StackFrame;
use crate::mm::pmm;
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
unsafe extern "C" fn excpt_non_maskable_interrupt(_stackframe: *mut StackFrame) -> *mut StackFrame {
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
unsafe extern "C" fn excpt_bound_range_exceeded(_stackframe: *mut StackFrame) -> *mut StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "C" fn excpt_invalid_opcode(_stackframe: *mut StackFrame) -> *mut StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "C" fn excpt_device_not_available(_stackframe: *mut StackFrame) -> *mut StackFrame {
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
unsafe extern "C" fn excpt_segment_not_present(_stackframe: *mut StackFrame) -> *mut StackFrame {
    unimplemented!()
}

#[no_mangle]
unsafe extern "C" fn excpt_stack_segment_fault(_stackframe: *mut StackFrame) -> *mut StackFrame {
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
    debug_assert!(adr != 0, "nullptr");
    if let Some(mut entry) = vm::get_page_entry(vm::get_cur(), adr) {
        let entry = entry.as_mut();
        if entry.resv() {
            entry.set_adr(pmm::alloc_pages_zeroed(1).phys_adr());
            entry.set_p(true);
        } else {
            panic!("accessed an unreserved page at adr '0x{adr:016x}'");
        }
    } else {
        panic!("invalid memory access at adr '0x{adr:016x}'!")
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
unsafe extern "C" fn excpt_simd_floating_point(_stackframe: *mut StackFrame) -> *mut StackFrame {
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
unsafe extern "C" fn excpt_hypervisor_injection(_stackframe: *mut StackFrame) -> *mut StackFrame {
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
