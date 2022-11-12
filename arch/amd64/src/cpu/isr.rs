#![allow(unreachable_code)]

pub use crate::{chipset::pic, cpu::idt};
use core::fmt::Display;

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
    pub fn pit() {}

    pub fn kbd() {
        unimplemented!()
    }

    pub fn cascade() {
        unimplemented!()
    }

    pub fn com2() {
        unimplemented!()
    }

    pub fn com1() {
        unimplemented!()
    }

    pub fn lpt2() {
        unimplemented!()
    }

    pub fn floppy() {
        unimplemented!()
    }

    pub fn lpt1() {
        unimplemented!()
    }

    pub fn cmos_rtc() {
        unimplemented!()
    }

    pub fn free0() {
        unimplemented!()
    }

    pub fn free1() {
        unimplemented!()
    }

    pub fn free2() {
        unimplemented!()
    }

    pub fn mouse() {
        unimplemented!()
    }

    pub fn cp() {
        unimplemented!()
    }

    pub fn primary_disk() {
        unimplemented!()
    }

    pub fn secondary_disk() {
        unimplemented!()
    }
}

mod exception {
    pub fn pg_fault() {
        unimplemented!()
    }
}

macro_rules! irqs {
    ($($irq:ident : $id:literal => fn { $($stmt:stmt);* $(;)? };)*) => {
        $(
            proc_macro::x86_isr_def! { irq $irq : $id => __handle_irq_ : fn { $($stmt);* } }
        )*

        pub fn init_irqs(start_vec: u8){
            $(
                idt::set_descriptor(start_vec + $id, $irq);
            )*
        }
    };
}

macro_rules! excepts {
    ($($except:ident : $id:literal => fn { $($stmt:stmt);* $(;)? };)*) => {
        $(
            proc_macro::x86_isr_def! { except $except : $id => __handle_except_ : fn { $($stmt);* } }
        )*

        pub fn init_excepts(start_vec: u8){
            $(
                idt::set_descriptor(start_vec + $id, $except);
            )*
        }
    };
}

irqs! {
    programmable_interrupt_timer : 0 => fn {
        irq::pit();
    };
    keyboard : 1 => fn {
        irq::kbd();
    };
    cascade : 2 => fn {
        irq::cascade();
    };
    com2 : 3 => fn {
        irq::com2();
    };
    com1 : 4 => fn {
        irq::com1();
    };
    lpt2 : 5 => fn {
        irq::lpt2();
    };
    floppy_disk : 6 => fn {
        irq::floppy();
    };
    lpt1 : 7 => fn {
        irq::lpt1();
    };
    cmos_rtc : 8 => fn{
        irq::cmos_rtc();
    };
    free_for_peripherals0 : 9 => fn{
        irq::free0();
    };
    free_for_peripherals1 : 10 => fn{
        irq::free1();
    };
    free_for_peripherals2 : 11 => fn{
        irq::free2();
    };
    ps2_mouse : 12 => fn{
        irq::mouse();
    };
    coprocessor: 13 => fn{
        irq::cp();
    };
    primary_ata_disk: 14 => fn{
        irq::primary_disk();
    };
    secondary_ata_disk: 15 => fn{
        irq::secondary_disk();
    };
}

excepts! {
    divide_by_zero : 0 => fn{
        panic!("divide by zero")
    };
    debug : 1 => fn{
        panic!("debug")
    };
    non_maskable_interrupt : 2 => fn{
        panic!("non maskable interrupt")
    };
    breakpoint : 3 => fn{
        panic!("breakpoint")
    };
    overflow : 4 => fn{
        panic!("overflow")
    };
    bound_range_exceeded : 5 => fn{
        panic!("bound range exceeded")
    };
    invalid_opcode : 6 => fn{
        panic!("invalid opcode")
    };
    device_not_available : 7 => fn{
        panic!("device not available")
    };
    double_fault : 8 => fn{
        panic!("double fault")
    };
    coprocessor_segment_overrun : 9 => fn{
        panic!("coprocessor segment overrun")
    };
    invalid_tss : 10 => fn{
        panic!("invalid tss")
    };
    segment_not_present : 11 => fn{
        panic!("segment not present")
    };
    stack_segment_fault : 12 => fn{
        panic!("stack segment fault")
    };
    general_protection_fault : 13 => fn{
        panic!("general protection fault")
    };
    page_fault : 14 => fn{
        exception::pg_fault();
    };
    reserved_15 : 15 => fn{
        panic!("reserved 0xF")
    };
    floating_point_exception_x87 : 16 => fn{
        panic!("x87 floating-point exception")
    };
    alignent_check : 17 => fn{
        panic!("alignment check")
    };
    machine_check : 18 => fn{
        panic!("machine check")
    };
    floating_point_exception_simd : 19 => fn{
        panic!("simd floating-point exception")
    };
    virtualization_exception : 20 => fn{
        panic!("virtualization exception")
    };
    control_protection_exception : 21 => fn{
        panic!("control protection exception")
    };
    reserved_22 : 22 => fn{
        panic!("reserved 0x16");
    };
    reservec_23 : 23 => fn{
        panic!("reserved 0x17");
    };
    reserved_24 : 24 => fn{
        panic!("reserved 0x18");
    };
    reserved_25 : 25 => fn{
        panic!("reserved 0x19");
    };
    reserved_26 : 26 => fn{
        panic!("reserved 0x1A");
    };
    reserved_27 : 27 => fn{
        panic!("reserved 0x1B");
    };
    hypervisor_injection_exception : 28 => fn{
        panic!("hypervisor injection exception")
    };
    vmm_communication_exception : 29 => fn{
        panic!("vmm communication exception")
    };
    security_exception : 30 => fn{
        panic!("security exception")
    };
    reserved_31 : 31 => fn{
        panic!("reserved 0x1F")
    };
}
