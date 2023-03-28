use core::arch::{asm, global_asm};
use core::mem::size_of;

use crate::mm::pmm;

global_asm!(include_str!("isr.s"));

type InterruptFn = fn();

bitfield! {
    #[derive(Clone, Copy)]
    struct IdtEntry(u128) {
        offset_lo: u64 @ 0..=15,
        offset_mid: u64 @ 16..=31,
        offset_hi: u64 @ 32..=63,
        segment_selector: u16 @ 0..=15,
        ist: u8 @ 32..=34,
        gate_type: u8 @ 40..=43,
        dpl: u8 @ 45..=46,
        p: bool @ 47,
    }
}

impl IdtEntry {
    fn zero() -> Self {
        Self(0)
    }

    fn offset(&self) -> u64 {
        self.offset_lo() | (self.offset_mid() << 16) | (self.offset_hi() << 32)
    }

    fn set_offset(&mut self, offs: u64) {
        let lo = offs & 0xFFFF;
        let mid = (offs >> 16) & 0xFFFF;
        let hi = offs >> 32;
        self.set_offset_lo(lo);
        self.set_offset_mid(mid);
        self.set_offset_hi(hi);
    }
}

#[repr(C, align(16))]
pub struct Idt {
    entries: [IdtEntry; 256],
}

impl Idt {
    pub fn new() -> Self {
        let mut idt = Idt {
            entries: [IdtEntry(0); 256],
        };
        macro_rules! entry {
            ($index:literal, $fn:ident) => {
                idt.set_entry($index, {
                    let mut entry = IdtEntry(0);
                    entry.set_offset(&$fn as *const _ as u64);
                    entry
                })
            };
        }
        use super::isr::*;
        entry!(0, _irq_handler_0);
        entry!(1, _irq_handler_1);
        entry!(2, _irq_handler_2);
        entry!(3, _irq_handler_3);
        entry!(4, _irq_handler_4);
        entry!(5, _irq_handler_5);
        entry!(6, _irq_handler_6);
        entry!(7, _irq_handler_7);
        entry!(8, _irq_handler_8);
        entry!(9, _irq_handler_9);
        entry!(10, _irq_handler_10);
        entry!(11, _irq_handler_11);
        entry!(12, _irq_handler_12);
        entry!(13, _irq_handler_13);
        entry!(14, _irq_handler_14);
        entry!(15, _irq_handler_15);
        entry!(16, _irq_handler_16);
        entry!(17, _irq_handler_17);
        entry!(18, _irq_handler_18);
        entry!(19, _irq_handler_19);
        entry!(20, _irq_handler_20);
        entry!(21, _irq_handler_21);
        entry!(22, _irq_handler_22);
        entry!(23, _irq_handler_23);
        entry!(24, _irq_handler_24);
        entry!(25, _irq_handler_25);
        entry!(26, _irq_handler_26);
        entry!(27, _irq_handler_27);
        entry!(28, _irq_handler_28);
        entry!(29, _irq_handler_29);
        entry!(30, _irq_handler_30);
        entry!(31, _irq_handler_31);
        for i in 32..=255 {
            idt.set_entry(i, {
                let mut entry = IdtEntry(0);
                entry.set_offset(&unimp as *const _ as u64);
                entry
            });
        }
        idt
    }

    pub unsafe fn install(&self) {
        #[repr(C)]
        struct IDTR {
            size: u16,
            offset: u64,
        }
        let idtr = IDTR {
            size: size_of::<Idt>() as u16 - 1,
            offset: self as *const _ as u64,
        };
        asm!(
            "lidt [rax]",
            in("rax") &idtr as *const _ as u64,
        )
    }

    fn set_entry(&mut self, index: u8, entry: IdtEntry) {
        self.entries[index as usize] = entry;
    }
}

#[ctor(core)]
unsafe fn init() {
    debug!("installing idt");
    let idt = pmm::alloc_pages(size_of::<Idt>().div_ceil(pmm::PAGE_SIZE)) as *mut Idt;
    (*idt) = Idt::new();
    (*idt).install();
}
