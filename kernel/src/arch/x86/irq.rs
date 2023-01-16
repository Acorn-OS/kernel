use crate::arch::{
    self,
    x86::{
        io::{in8_delay, out8_delay},
        segments,
    },
};
use core::arch::asm;
use core::{arch::global_asm, fmt::Display};

const PIC1_BASE: u16 = 0x20;
const PIC2_BASE: u16 = 0xA0;

const COMMAND_OFFSET: u16 = 0;
const DATA_OFFSET: u16 = 1;

/// Different commands to send to the PIC.
mod cmd {
    /// End of interrupt command.
    pub const EOI: u8 = 0x20;
    /// TODO:
    pub const ICW1_INIT: u8 = 0x10;
    /// TODO:
    pub const ICW4_8086: u8 = 0x01;
}

struct Pic {
    base: u16,
    cascade_ident: u8,
}

impl Pic {
    fn remap(&self, start_vec: u8) {
        let mask = self.data_in();
        self.command(cmd::ICW1_INIT);
        self.data_out(start_vec);
        self.data_out(self.cascade_ident);
        self.data_out(cmd::ICW4_8086);
        self.data_out(mask);
    }

    #[inline]
    fn command(&self, cmd: u8) {
        out8_delay(self.base + COMMAND_OFFSET, cmd);
    }

    #[inline]
    fn data_out(&self, out: u8) {
        out8_delay(self.base + DATA_OFFSET, out);
    }

    #[inline]
    fn data_in(&self) -> u8 {
        in8_delay(self.base + DATA_OFFSET)
    }
}

static PIC1: Pic = Pic {
    base: PIC1_BASE,
    cascade_ident: 4,
};
static PIC2: Pic = Pic {
    base: PIC2_BASE,
    cascade_ident: 2,
};

/// Remaps both the PIC1 and PIC2.
fn pic_remap(pic1_vec: u8, pic2_vec: u8) {
    pic1::remap(pic1_vec);
    pic2::remap(pic2_vec);
}

macro_rules! impl_pic {
    ($ident:ident) => {
        use super::*;

        /// Remaps the PIC from the starting vector.
        pub fn remap(start_vec: u8) {
            $ident.remap(start_vec);
        }

        /// Masks the PIC.
        pub fn mask(mask: u8) {
            $ident.data_out(mask);
        }

        /// Sends a command to the PIC.
        pub fn cmd(cmd: u8) {
            $ident.command(cmd);
        }

        /// Signals the end of an interrupt for the PIC.
        pub fn end_of_interrupt() {
            cmd(cmd::EOI)
        }

        /// Enable all interrupts.
        pub fn enable_all() {
            mask(0);
        }

        /// Disable all interrupts.
        pub fn disable_all() {
            mask(0xFF);
        }
    };
}

/// Operate on PIC1.
mod pic1 {
    impl_pic!(PIC1);
}

/// Operator on PIC2.
mod pic2 {
    impl_pic!(PIC2);
}

#[allow(dead_code)]
#[repr(u8)]
enum Attribute {
    TaskGate = 0b0101,
    TrapGate16 = 0b0111,
    TrapGate32 = 0b1111,
    IntGate16 = 0b0110,
    IntGate32 = 0b1110,
    DPL0 = 0,
    DPL1 = 0b01_00000,
    DPL2 = 0b10_00000,
    DPL3 = 0b11_00000,
    Present = 0b1000_0000,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct IDTEntry {
    offset_1: u16,
    selector: u16,
    ist: u8,
    type_attributes: u8,
    offset_2: u16,
    offset_3: u32,
    _zero: u32,
}

impl IDTEntry {
    const fn new_null() -> Self {
        Self {
            offset_1: 0,
            selector: 0,
            ist: 0,
            type_attributes: 0,
            offset_2: 0,
            offset_3: 0,
            _zero: 0,
        }
    }
}

#[repr(C, packed)]
struct IDTR {
    size: u16,
    paged_offset: u64,
}

impl IDTR {
    fn new(idt: &IDT) -> Self {
        Self {
            size: core::mem::size_of::<IDT>() as u16,
            paged_offset: idt as *const _ as u64,
        }
    }
}

struct IDT {
    entries: [IDTEntry; 256],
}

impl IDT {
    const fn new() -> Self {
        Self {
            entries: [IDTEntry::new_null(); 256],
        }
    }
}

static mut IDT: IDT = IDT::new();

/// Sets a descriptor in the IDT.
unsafe fn idt_set_descriptor(index: u8, interrupt_handler: unsafe extern "C" fn()) {
    idt_set_entry(index, interrupt_handler as u64);
}

/// Installs the IDT.
unsafe fn idt_install() {
    let idtr = IDTR::new(&IDT);
    let idtr_ptr = &idtr as *const IDTR as u64;
    unsafe {
        asm!(
            "lidt [rax]",
            in("rax")(idtr_ptr)
        )
    }
}

unsafe fn idt_set_entry(entry: u8, val: u64) {
    let entry = &mut IDT.entries[entry as usize];
    entry.offset_1 = val as u16;
    entry.offset_2 = (val >> 16) as u16;
    entry.offset_3 = (val >> 32) as u32;
    entry.selector = segments::KERNEL_CODE_SELECTOR;
    entry.ist = 0;
    entry.type_attributes = Attribute::Present as u8 | Attribute::IntGate32 as u8;
    entry._zero = 0;
}

static _ASSERT_ENTRY_SIZE: () = assert!(core::mem::size_of::<IDTEntry>() == 16);
static _ASSERT_IDT_SIZE: () = assert!(core::mem::size_of::<IDT>() == 16 * 256);
static _ASSERT_IDR_SIZE: () = assert!(core::mem::size_of::<IDTR>() == 10);

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

/// Signals the end of an interrupt for both PICs.
/// This is preferable over signaling to indiviual
/// PICs.
#[inline]
fn end_of_interrupt() {
    pic1::end_of_interrupt();
    pic2::end_of_interrupt();
}

mod irq {
    use super::{end_of_interrupt, StackFrame};

    #[no_mangle]
    pub extern "C" fn programmable_interrupt_timer(_stack_frame: *mut StackFrame) {
        end_of_interrupt();
    }

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

global_asm!(include_str!("isr.s"));

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

/// Disables interrupts for both PIC1 and PIC2.
fn mask_all() {
    pic1::disable_all();
    pic2::disable_all();
}

/// Enables interrupts for both PIC1 and PIC2.
fn unmask_all() {
    pic1::enable_all();
    pic2::enable_all();
}

pub unsafe fn init() {
    idt_set_descriptor(IRQ_START_VEC + 0, _irq_handler_0);
    idt_set_descriptor(IRQ_START_VEC + 1, _irq_handler_1);
    idt_set_descriptor(IRQ_START_VEC + 2, _irq_handler_2);
    idt_set_descriptor(IRQ_START_VEC + 3, _irq_handler_3);
    idt_set_descriptor(IRQ_START_VEC + 4, _irq_handler_4);
    idt_set_descriptor(IRQ_START_VEC + 5, _irq_handler_5);
    idt_set_descriptor(IRQ_START_VEC + 6, _irq_handler_6);
    idt_set_descriptor(IRQ_START_VEC + 7, _irq_handler_7);
    idt_set_descriptor(IRQ_START_VEC + 8, _irq_handler_8);
    idt_set_descriptor(IRQ_START_VEC + 9, _irq_handler_9);
    idt_set_descriptor(IRQ_START_VEC + 10, _irq_handler_10);
    idt_set_descriptor(IRQ_START_VEC + 11, _irq_handler_11);
    idt_set_descriptor(IRQ_START_VEC + 12, _irq_handler_12);
    idt_set_descriptor(IRQ_START_VEC + 13, _irq_handler_13);
    idt_set_descriptor(IRQ_START_VEC + 14, _irq_handler_14);
    idt_set_descriptor(IRQ_START_VEC + 15, _irq_handler_15);

    idt_set_descriptor(EXCEPT_START_VEC + 0, _except_handler_0);
    idt_set_descriptor(EXCEPT_START_VEC + 1, _except_handler_1);
    idt_set_descriptor(EXCEPT_START_VEC + 2, _except_handler_2);
    idt_set_descriptor(EXCEPT_START_VEC + 3, _except_handler_3);
    idt_set_descriptor(EXCEPT_START_VEC + 4, _except_handler_4);
    idt_set_descriptor(EXCEPT_START_VEC + 5, _except_handler_5);
    idt_set_descriptor(EXCEPT_START_VEC + 6, _except_handler_6);
    idt_set_descriptor(EXCEPT_START_VEC + 7, _except_handler_7);
    idt_set_descriptor(EXCEPT_START_VEC + 8, _except_handler_8);
    idt_set_descriptor(EXCEPT_START_VEC + 9, _except_handler_9);
    idt_set_descriptor(EXCEPT_START_VEC + 10, _except_handler_10);
    idt_set_descriptor(EXCEPT_START_VEC + 11, _except_handler_11);
    idt_set_descriptor(EXCEPT_START_VEC + 12, _except_handler_12);
    idt_set_descriptor(EXCEPT_START_VEC + 13, _except_handler_13);
    idt_set_descriptor(EXCEPT_START_VEC + 14, _except_handler_14);
    idt_set_descriptor(EXCEPT_START_VEC + 15, _except_handler_15);
    idt_set_descriptor(EXCEPT_START_VEC + 16, _except_handler_16);
    idt_set_descriptor(EXCEPT_START_VEC + 17, _except_handler_17);
    idt_set_descriptor(EXCEPT_START_VEC + 18, _except_handler_18);
    idt_set_descriptor(EXCEPT_START_VEC + 19, _except_handler_19);
    idt_set_descriptor(EXCEPT_START_VEC + 20, _except_handler_20);
    idt_set_descriptor(EXCEPT_START_VEC + 21, _except_handler_21);
    idt_set_descriptor(EXCEPT_START_VEC + 22, _except_handler_22);
    idt_set_descriptor(EXCEPT_START_VEC + 23, _except_handler_23);
    idt_set_descriptor(EXCEPT_START_VEC + 24, _except_handler_24);
    idt_set_descriptor(EXCEPT_START_VEC + 25, _except_handler_25);
    idt_set_descriptor(EXCEPT_START_VEC + 26, _except_handler_26);
    idt_set_descriptor(EXCEPT_START_VEC + 27, _except_handler_27);
    idt_set_descriptor(EXCEPT_START_VEC + 28, _except_handler_28);
    idt_set_descriptor(EXCEPT_START_VEC + 29, _except_handler_29);
    idt_set_descriptor(EXCEPT_START_VEC + 30, _except_handler_30);
    idt_set_descriptor(EXCEPT_START_VEC + 31, _except_handler_31);

    pic_remap(IRQ_START_VEC, IRQ_START_VEC + 8);
    mask_all();
    idt_install();
}
