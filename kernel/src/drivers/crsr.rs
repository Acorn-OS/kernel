//! cursor driver

use crate::arch::interrupt;

pub extern "C" fn main() -> ! {
    trace!("running cursor driver");
    loop {
        trace!("cursor");
        interrupt::halt()
    }
}
