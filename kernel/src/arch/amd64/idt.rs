use crate::mm::pmm;
use crate::mm::vmm::VirtualMemory;

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
    struct IdtEntry(u128) {
        offset_lo: u64 @ 0..=15,
        offset_hi: u64 @ 48..=95,
        segment_selector: u16 @ 16..=31,
        ist: u8 @ 32..=34,
        gate_type: u8 @ 40..=43,
        dpl: u8 @ 45..=46,
        p: bool @ 47,
    }
}

impl IdtEntry {
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
    entries: [IdtEntry; 256],
}

impl Idt {
    pub fn new() -> Self {
        let mut idt = Idt {
            entries: [IdtEntry(0); 256],
        };
        macro_rules! except {
            ($index:literal, $func:ident) => {
                idt.set_entry($index, {
                    let mut entry = IdtEntry(0);
                    entry.set_offset($func as u64);
                    entry.set_p(true);
                    entry.set_segment_selector(KERNEL_CODE_SELECTOR);
                    entry.set_gate(GateType::Trap);
                    entry
                })
            };
        }
        macro_rules! int {
            ($index:literal, $func:ident) => {
                idt.set_entry($index, {
                    let mut entry = IdtEntry(0);
                    entry.set_offset($func as u64);
                    entry.set_p(true);
                    entry.set_segment_selector(KERNEL_CODE_SELECTOR);
                    entry.set_gate(GateType::Int);
                    entry
                })
            };
        }
        use super::isr::*;
        except!(0, irq_handler_0);
        except!(1, irq_handler_1);
        except!(2, irq_handler_2);
        except!(3, irq_handler_3);
        except!(4, irq_handler_4);
        except!(5, irq_handler_5);
        except!(6, irq_handler_6);
        except!(7, irq_handler_7);
        except!(8, irq_handler_8);
        except!(9, irq_handler_9);
        except!(10, irq_handler_10);
        except!(11, irq_handler_11);
        except!(12, irq_handler_12);
        except!(13, irq_handler_13);
        except!(14, irq_handler_14);
        except!(15, irq_handler_15);
        except!(16, irq_handler_16);
        except!(17, irq_handler_17);
        except!(18, irq_handler_18);
        except!(19, irq_handler_19);
        except!(20, irq_handler_20);
        except!(21, irq_handler_21);
        except!(22, irq_handler_22);
        except!(23, irq_handler_23);
        except!(24, irq_handler_24);
        except!(25, irq_handler_25);
        except!(26, irq_handler_26);
        except!(27, irq_handler_27);
        except!(28, irq_handler_28);
        except!(29, irq_handler_29);
        except!(30, irq_handler_30);
        except!(31, irq_handler_31);
        int!(32, irq_handler_32);
        int!(33, irq_handler_33);
        int!(34, irq_handler_34);
        int!(35, irq_handler_35);
        int!(36, irq_handler_36);
        int!(37, irq_handler_37);
        int!(38, irq_handler_38);
        int!(39, irq_handler_39);
        int!(40, irq_handler_40);
        int!(41, irq_handler_41);
        int!(42, irq_handler_42);
        int!(43, irq_handler_43);
        int!(44, irq_handler_44);
        int!(45, irq_handler_45);
        int!(46, irq_handler_46);
        int!(47, irq_handler_47);
        for i in 48..=255 {
            idt.set_entry(i, {
                let mut entry = IdtEntry(0);
                entry.set_offset(&unimp as *const _ as u64);
                entry.set_p(true);
                entry.set_segment_selector(KERNEL_CODE_SELECTOR);
                entry.set_gate(GateType::Int);
                entry
            });
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

    fn set_entry(&mut self, index: u8, entry: IdtEntry) {
        self.entries[index as usize] = entry;
    }
}

pub unsafe fn new(map: &mut VirtualMemory) -> *mut Idt {
    let pages = size_of::<Idt>().div_ceil(pmm::PAGE_SIZE);
    let ptr = map.map_pages(pages, pmm::alloc_pages(pages) as u64) as *mut Idt;
    ptr.write(Idt::new());
    ptr
}
