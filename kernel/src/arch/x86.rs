pub mod fb;
pub mod mm;

mod boot;
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
