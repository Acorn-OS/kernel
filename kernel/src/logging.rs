use crate::arch::serial::uart;
use core::fmt::Write;

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        true
    }

    #[allow(unused_must_use)]
    fn log(&self, record: &log::Record) {
        let mut_self = unsafe { &mut *(self as *const Self as *mut Self) };
        match record.level() {
            log::Level::Error => {
                write!(
                    mut_self,
                    "{} {}:",
                    record.file().unwrap(),
                    record.line().unwrap()
                );
                write!(mut_self, "[ERR] {}\n", record.args());
            }
            log::Level::Warn => todo!(),
            log::Level::Info => {
                write!(mut_self, "{}\n", record.args());
            }
            log::Level::Debug => {
                write!(mut_self, "{}\n", record.args());
            }
            log::Level::Trace => {
                write!(mut_self, "{}\n", record.args());
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
