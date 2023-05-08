use super::gdt::KERNEL_CODE_SELECTOR;
use core::arch::asm;
use core::mem::size_of;

#[repr(u8)]
enum GateType {
    Int = 0xE,
    Trap = 0xF,
}

bitfield! {
    #[derive(Clone, Copy)]
    struct Entry(u128) {
        offset_lo: u64 @ 0..=15,
        offset_hi: u64 @ 48..=95,
        segment_selector: u16 @ 16..=31,
        ist: u8 @ 32..=34,
        gate_type: u8 @ 40..=43,
        dpl: u8 @ 45..=46,
        p: bool @ 47,
    }
}

impl Entry {
    fn offset(&self) -> u64 {
        self.offset_lo() | (self.offset_hi() << 16)
    }

    fn set_offset(&mut self, offs: u64) {
        let lo = offs & 0xFFFF;
        let hi = offs >> 16;
        self.set_offset_lo(lo);
        self.set_offset_hi(hi);
    }

    fn set_gate(&mut self, ty: GateType) {
        self.set_gate_type(ty as u8);
    }
}

#[repr(C, align(16))]
pub struct Idt {
    entries: [Entry; 256],
}

impl Idt {
    pub fn new() -> Self {
        let mut idt = Idt {
            entries: [Entry(0); 256],
        };
        for (i, f) in unsafe { super::interrupt::irq_routines }
            .iter()
            .cloned()
            .enumerate()
        {
            idt.set_entry(i as u8, {
                let mut entry = Entry(0);
                entry.set_offset(f as u64);
                entry.set_p(true);
                entry.set_segment_selector(KERNEL_CODE_SELECTOR);
                entry.set_ist(0);
                if i < 32 {
                    entry.set_gate(GateType::Trap);
                } else {
                    entry.set_gate(GateType::Int);
                }
                entry
            })
        }
        idt
    }

    pub unsafe fn install(&self) {
        #[repr(C, packed)]
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

    fn set_entry(&mut self, index: u8, entry: Entry) {
        self.entries[index as usize] = entry;
    }
}

static mut IDT: Idt = Idt {
    entries: [Entry(0); 256],
};

pub unsafe fn init() {
    IDT = Idt::new();
}

pub unsafe fn install() {
    IDT.install()
}
