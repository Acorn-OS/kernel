pub mod fb;
pub mod mm;

mod cpu;
mod io;
mod irq;
mod segments;
mod serial;

pub fn log(s: &str) {
    for c in s.chars() {
        serial::putb(c as u8)
    }
}

pub unsafe fn init() {
    serial::init();
    trace!("pre init.");
    trace!("vm init.");
    mm::vm::init();
    trace!("init segmentation.");
    segments::init();
    trace!("init irq/exception.");
    irq::init();
    trace!("init framebuffer.");
    fb::init();
}
