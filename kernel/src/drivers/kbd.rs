//! keyboard driver

use crate::arch::interrupt;

pub extern "C" fn main() -> ! {
    trace!("running keyboard driver");
    loop {
        trace!("keyboard");
        interrupt::halt();
    }
}
