use core::fmt::Write;

use crate::arch;

struct PanicWriter;

static mut PANIC_WRITER: PanicWriter = PanicWriter;

impl Write for PanicWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        arch::log(s);
        Ok(())
    }
}

#[allow(unused_must_use)]
#[panic_handler]
pub unsafe fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    let location = info.location().unwrap();
    PANIC_WRITER.write_fmt(format_args!(
        "{} {}:{}\n\r{}\n\r",
        location.file(),
        location.line(),
        location.column(),
        info.message().unwrap()
    ));
    loop {}
}
