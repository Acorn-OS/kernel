use crate::arch::serial::uart;
use core::fmt::Write;

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    #[allow(unused_must_use)]
    fn log(&self, record: &log::Record) {
        let mut_self = unsafe { &mut *(self as *const Self as *mut Self) };
        match record.level() {
            log::Level::Error => {
                writeln!(
                    mut_self,
                    "\n{}:{}\n{}",
                    record.file().unwrap(),
                    record.line().unwrap(),
                    record.args()
                );
            }
            log::Level::Warn => {
                writeln!(mut_self, "[W] {}", record.args());
            }
            log::Level::Info => {
                writeln!(mut_self, "{}", record.args());
            }
            log::Level::Debug => {
                writeln!(mut_self, "[D] {}", record.args());
            }
            log::Level::Trace => {
                writeln!(mut_self, "[T] {}", record.args());
            }
        };
    }

    fn flush(&self) {}
}

impl Write for Logger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        uart::puts(s);
        Ok(())
    }
}

static LOGGER: Logger = Logger;

pub fn init() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::max());
}
