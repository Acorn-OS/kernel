#![allow(unreachable_code)]

use crate::{
    chipset,
    cpu::{self, idt},
};
use core::{arch::global_asm, fmt::Display};

global_asm!(include_str!("isr.s"));

/// A stack frame for retaining register values
/// throughout interrupts.
#[repr(C, packed)]
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
    rdi: i64,
    rsi: u64,
    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64,
    id: u64,
    rip: u64,
    cs: u64,
    rf: u64,
    rsp: u64,
    ss: u64,
}

impl Display for StackFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self {
            ss,
            rsp,
            rf,
            cs,
            rip,
            id,
            rax,
            rbx,
            rcx,
            rdx,
            rsi,
            rdi,
            r8,
            r9,
            r10,
            r11,
            r12,
            r13,
            r14,
            r15,
            rbp,
        } = *self;
        f.write_fmt(format_args!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
            format_args!("ss:  {ss:016X}"),
            format_args!("rsp: {rsp:016X}"),
            format_args!("rf:  {rf:016X}"),
            format_args!("cs:  {cs:016X}"),
            format_args!("rip: {rip:016X}"),
            format_args!("id:  {id:016X}"),
            format_args!("rax: {rax:016X}"),
            format_args!("rbx: {rbx:016X}"),
            format_args!("rcx: {rcx:016X}"),
            format_args!("rdx: {rdx:016X}"),
            format_args!("rsi: {rsi:016X}"),
            format_args!("rdi: {rdi:016X}"),
            format_args!("r8:  {r8:016X}"),
            format_args!("r9:  {r9:016X}"),
            format_args!("r10: {r10:016X}"),
            format_args!("r11: {r11:016X}"),
            format_args!("r12: {r12:016X}"),
            format_args!("r13: {r13:016X}"),
            format_args!("r14: {r14:016X}"),
            format_args!("r15: {r15:016X}"),
            format_args!("rbp: {rbp:016X}"),
        ))
    }
}

mod irq {
    use super::StackFrame;

    #[no_mangle]
    pub extern "C" fn programmable_interrupt_timer(_stack_frame: *mut StackFrame) {}

    #[no_mangle]
    pub extern "C" fn keyboard(_stack_frame: *mut StackFrame) {
        panic!("kbd")
    }

    #[no_mangle]
    pub extern "C" fn cascade(_stack_frame: *mut StackFrame) {
        panic!("cascade")
    }

    #[no_mangle]
    pub extern "C" fn com2(_stack_frame: *mut StackFrame) {
        panic!("com2")
    }

    #[no_mangle]
    pub extern "C" fn com1(_stack_frame: *mut StackFrame) {
        panic!("com1")
    }

    #[no_mangle]
    pub extern "C" fn lpt2(_stack_frame: *mut StackFrame) {
        panic!("lpt2")
    }

    #[no_mangle]
    pub extern "C" fn floppy_disk(_stack_frame: *mut StackFrame) {
        panic!("floppy")
    }

    #[no_mangle]
    pub extern "C" fn lpt1(_stack_frame: *mut StackFrame) {
        panic!("lpt1")
    }

    #[no_mangle]
    pub extern "C" fn cmos_rtc(_stack_frame: *mut StackFrame) {
        panic!("cmos_rtc")
    }

    #[no_mangle]
    pub extern "C" fn free0(_stack_frame: *mut StackFrame) {
        panic!("free0")
    }

    #[no_mangle]
    pub extern "C" fn free1(_stack_frame: *mut StackFrame) {
        panic!("free1")
    }

    #[no_mangle]
    pub extern "C" fn free2(_stack_frame: *mut StackFrame) {
        panic!("free2")
    }

    #[no_mangle]
    pub extern "C" fn ps2_mouse(_stack_frame: *mut StackFrame) {
        panic!("mouse")
    }

    #[no_mangle]
    pub extern "C" fn coprocessor(_stack_frame: *mut StackFrame) {
        panic!("cp")
    }

    #[no_mangle]
    pub extern "C" fn primary_ata_disk(_stack_frame: *mut StackFrame) {
        panic!("primary disk")
    }

    #[no_mangle]
    pub extern "C" fn secondary_ata_disk(_stack_frame: *mut StackFrame) {
        panic!("secondary_disk")
    }
}

mod exception {
    use super::StackFrame;

    #[no_mangle]
    pub extern "C" fn divide_by_zero(_stack_frame: *mut StackFrame) {
        panic!("divide by zero")
    }

    #[no_mangle]
    pub extern "C" fn debug(_stack_frame: *mut StackFrame) {
        panic!("debug")
    }

    #[no_mangle]
    pub extern "C" fn non_maskable_interrupt(_stack_frame: *mut StackFrame) {
        panic!("non_maskable_interrupt")
    }

    #[no_mangle]
    pub extern "C" fn breakpoint(_stack_frame: *mut StackFrame) {
        panic!("breakpoint")
    }

    #[no_mangle]
    pub extern "C" fn overflow(_stack_frame: *mut StackFrame) {
        panic!("overflow")
    }

    #[no_mangle]
    pub extern "C" fn bound_range_exceeded(_stack_frame: *mut StackFrame) {
        panic!("bound_range_exceeded")
    }

    #[no_mangle]
    pub extern "C" fn invalid_opcode(_stack_frame: *mut StackFrame) {
        panic!("invalid_opcode")
    }

    #[no_mangle]
    pub extern "C" fn device_not_available(_stack_frame: *mut StackFrame) {
        panic!("device_not_available")
    }

    #[no_mangle]
    pub extern "C" fn double_fault(_stack_frame: *mut StackFrame) {
        panic!("double_fault")
    }

    #[no_mangle]
    pub extern "C" fn coprocessor_segment_overrun(_stack_frame: *mut StackFrame) {
        panic!("coprocessor_segment_overrun")
    }

    #[no_mangle]
    pub extern "C" fn invalid_tss(_stack_frame: *mut StackFrame) {
        panic!("invalid_tss")
    }

    #[no_mangle]
    pub extern "C" fn segment_not_present(_stack_frame: *mut StackFrame) {
        panic!("segment_not_present")
    }

    #[no_mangle]
    pub extern "C" fn stack_segment_fault(_stack_frame: *mut StackFrame) {
        panic!("stack_segment_fault")
    }

    #[no_mangle]
    pub extern "C" fn general_protection_fault(_stack_frame: *mut StackFrame) {
        panic!("general_protection_fault")
    }

    #[no_mangle]
    pub extern "C" fn page_fault(_stack_frame: *mut StackFrame) {
        panic!("page_fault")
    }

    #[no_mangle]
    pub extern "C" fn reserved_15(_stack_frame: *mut StackFrame) {
        panic!("reserved_15")
    }

    #[no_mangle]
    pub extern "C" fn floating_point_exception_x87(_stack_frame: *mut StackFrame) {
        panic!("floating_point_exception_x87")
    }

    #[no_mangle]
    pub extern "C" fn alignent_check(_stack_frame: *mut StackFrame) {
        panic!("alignent_check")
    }

    #[no_mangle]
    pub extern "C" fn machine_check(_stack_frame: *mut StackFrame) {
        panic!("machine_check")
    }

    #[no_mangle]
    pub extern "C" fn floating_point_exception_simd(_stack_frame: *mut StackFrame) {
        panic!("floating_point_exception_simd")
    }

    #[no_mangle]
    pub extern "C" fn virtualization_exception(_stack_frame: *mut StackFrame) {
        panic!("virtualization_exception")
    }

    #[no_mangle]
    pub extern "C" fn control_protection_exception(_stack_frame: *mut StackFrame) {
        panic!("control_protection_exception")
    }

    #[no_mangle]
    pub extern "C" fn reserved_22(_stack_frame: *mut StackFrame) {
        panic!("reserved_22")
    }

    #[no_mangle]
    pub extern "C" fn reservec_23(_stack_frame: *mut StackFrame) {
        panic!("reservec_23")
    }

    #[no_mangle]
    pub extern "C" fn reserved_24(_stack_frame: *mut StackFrame) {
        panic!("reserved_24")
    }

    #[no_mangle]
    pub extern "C" fn reserved_25(_stack_frame: *mut StackFrame) {
        panic!("reserved_25")
    }

    #[no_mangle]
    pub extern "C" fn reserved_26(_stack_frame: *mut StackFrame) {
        panic!("reserved_26")
    }

    #[no_mangle]
    pub extern "C" fn reserved_27(_stack_frame: *mut StackFrame) {
        panic!("reserved_27")
    }

    #[no_mangle]
    pub extern "C" fn hypervisor_injection_exception(_stack_frame: *mut StackFrame) {
        panic!("hypervisor_injection_exception")
    }

    #[no_mangle]
    pub extern "C" fn vmm_communication_exception(_stack_frame: *mut StackFrame) {
        panic!("vmm_communication_exception")
    }

    #[no_mangle]
    pub extern "C" fn security_exception(_stack_frame: *mut StackFrame) {
        panic!("security_exception")
    }

    #[no_mangle]
    pub extern "C" fn reserved_31(_stack_frame: *mut StackFrame) {
        panic!("reserved_31")
    }
}

extern "C" {
    fn _irq_handler_0();
    fn _irq_handler_1();
    fn _irq_handler_2();
    fn _irq_handler_3();
    fn _irq_handler_4();
    fn _irq_handler_5();
    fn _irq_handler_6();
    fn _irq_handler_7();
    fn _irq_handler_8();
    fn _irq_handler_9();
    fn _irq_handler_10();
    fn _irq_handler_11();
    fn _irq_handler_12();
    fn _irq_handler_13();
    fn _irq_handler_14();
    fn _irq_handler_15();

    fn _except_handler_0();
    fn _except_handler_1();
    fn _except_handler_2();
    fn _except_handler_3();
    fn _except_handler_4();
    fn _except_handler_5();
    fn _except_handler_6();
    fn _except_handler_7();
    fn _except_handler_8();
    fn _except_handler_9();
    fn _except_handler_10();
    fn _except_handler_11();
    fn _except_handler_12();
    fn _except_handler_13();
    fn _except_handler_14();
    fn _except_handler_15();
    fn _except_handler_16();
    fn _except_handler_17();
    fn _except_handler_18();
    fn _except_handler_19();
    fn _except_handler_20();
    fn _except_handler_21();
    fn _except_handler_22();
    fn _except_handler_23();
    fn _except_handler_24();
    fn _except_handler_25();
    fn _except_handler_26();
    fn _except_handler_27();
    fn _except_handler_28();
    fn _except_handler_29();
    fn _except_handler_30();
    fn _except_handler_31();
}

const IRQ_START_VEC: u8 = 0x20;
const EXCEPT_START_VEC: u8 = 0x00;

pub unsafe fn init() {
    idt::set_descriptor(IRQ_START_VEC + 0, _irq_handler_0);
    idt::set_descriptor(IRQ_START_VEC + 1, _irq_handler_1);
    idt::set_descriptor(IRQ_START_VEC + 2, _irq_handler_2);
    idt::set_descriptor(IRQ_START_VEC + 3, _irq_handler_3);
    idt::set_descriptor(IRQ_START_VEC + 4, _irq_handler_4);
    idt::set_descriptor(IRQ_START_VEC + 5, _irq_handler_5);
    idt::set_descriptor(IRQ_START_VEC + 6, _irq_handler_6);
    idt::set_descriptor(IRQ_START_VEC + 7, _irq_handler_7);
    idt::set_descriptor(IRQ_START_VEC + 8, _irq_handler_8);
    idt::set_descriptor(IRQ_START_VEC + 9, _irq_handler_9);
    idt::set_descriptor(IRQ_START_VEC + 10, _irq_handler_10);
    idt::set_descriptor(IRQ_START_VEC + 11, _irq_handler_11);
    idt::set_descriptor(IRQ_START_VEC + 12, _irq_handler_12);
    idt::set_descriptor(IRQ_START_VEC + 13, _irq_handler_13);
    idt::set_descriptor(IRQ_START_VEC + 14, _irq_handler_14);
    idt::set_descriptor(IRQ_START_VEC + 15, _irq_handler_15);

    idt::set_descriptor(EXCEPT_START_VEC + 0, _except_handler_0);
    idt::set_descriptor(EXCEPT_START_VEC + 1, _except_handler_1);
    idt::set_descriptor(EXCEPT_START_VEC + 2, _except_handler_2);
    idt::set_descriptor(EXCEPT_START_VEC + 3, _except_handler_3);
    idt::set_descriptor(EXCEPT_START_VEC + 4, _except_handler_4);
    idt::set_descriptor(EXCEPT_START_VEC + 5, _except_handler_5);
    idt::set_descriptor(EXCEPT_START_VEC + 6, _except_handler_6);
    idt::set_descriptor(EXCEPT_START_VEC + 7, _except_handler_7);
    idt::set_descriptor(EXCEPT_START_VEC + 8, _except_handler_8);
    idt::set_descriptor(EXCEPT_START_VEC + 9, _except_handler_9);
    idt::set_descriptor(EXCEPT_START_VEC + 10, _except_handler_10);
    idt::set_descriptor(EXCEPT_START_VEC + 11, _except_handler_11);
    idt::set_descriptor(EXCEPT_START_VEC + 12, _except_handler_12);
    idt::set_descriptor(EXCEPT_START_VEC + 13, _except_handler_13);
    idt::set_descriptor(EXCEPT_START_VEC + 14, _except_handler_14);
    idt::set_descriptor(EXCEPT_START_VEC + 15, _except_handler_15);
    idt::set_descriptor(EXCEPT_START_VEC + 16, _except_handler_16);
    idt::set_descriptor(EXCEPT_START_VEC + 17, _except_handler_17);
    idt::set_descriptor(EXCEPT_START_VEC + 18, _except_handler_18);
    idt::set_descriptor(EXCEPT_START_VEC + 19, _except_handler_19);
    idt::set_descriptor(EXCEPT_START_VEC + 20, _except_handler_20);
    idt::set_descriptor(EXCEPT_START_VEC + 21, _except_handler_21);
    idt::set_descriptor(EXCEPT_START_VEC + 22, _except_handler_22);
    idt::set_descriptor(EXCEPT_START_VEC + 23, _except_handler_23);
    idt::set_descriptor(EXCEPT_START_VEC + 24, _except_handler_24);
    idt::set_descriptor(EXCEPT_START_VEC + 25, _except_handler_25);
    idt::set_descriptor(EXCEPT_START_VEC + 26, _except_handler_26);
    idt::set_descriptor(EXCEPT_START_VEC + 27, _except_handler_27);
    idt::set_descriptor(EXCEPT_START_VEC + 28, _except_handler_28);
    idt::set_descriptor(EXCEPT_START_VEC + 29, _except_handler_29);
    idt::set_descriptor(EXCEPT_START_VEC + 30, _except_handler_30);
    idt::set_descriptor(EXCEPT_START_VEC + 31, _except_handler_31);

    chipset::pic::remap(IRQ_START_VEC, IRQ_START_VEC + 8);
    chipset::pic::enable_all();
    cpu::idt::install();
}
