use super::port::out8;

const PIC0_CMD: u16 = 0x20;
const PIC0_DATA: u16 = 0x21;
const PIC1_CMD: u16 = 0xa0;
const PIC1_DATA: u16 = 0xa1;

const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;
const ICW4_8086: u8 = 0x01;
const EOI: u8 = 0x20;

pub unsafe fn disable() {
    trace!("disabling PIC");
    // start initialization of the PICs.
    out8(PIC0_CMD, ICW1_INIT | ICW1_ICW4);
    out8(PIC1_CMD, ICW1_INIT | ICW1_ICW4);
    // mask the pic interrupts.
    out8(PIC0_DATA, 32);
    out8(PIC1_DATA, 32 + 8);
    out8(PIC0_DATA, 4);
    out8(PIC1_DATA, 2);
    out8(PIC0_DATA, ICW4_8086);
    out8(PIC1_DATA, ICW4_8086);
    // finally, disable the PIC.
    out8(PIC0_DATA, 0xFF);
    out8(PIC1_DATA, 0xFF);
    // signal end of interrupts to drop pending interrupts.
    out8(PIC0_CMD, EOI);
    out8(PIC1_CMD, EOI);
}
