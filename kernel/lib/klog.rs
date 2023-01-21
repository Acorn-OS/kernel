use core::fmt::Write;
use log::Log;

use super::arch;

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
                write!(mut_self, "[ERR] {}\n\r", record.args());
            }
            log::Level::Warn => todo!(),
            log::Level::Info => {
                write!(mut_self, "{}\n\r", record.args());
            }
            log::Level::Debug => {
                write!(mut_self, "{}\n\r", record.args());
            }
            log::Level::Trace => {
                write!(mut_self, "{}\n\r", record.args());
            }
        };
    }

    fn flush(&self) {}
}

impl Write for Logger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        arch::log(s);
        Ok(())
    }
}

pub fn log_simple(s: &str) {
    arch::log(s)
}

pub fn log(record: &log::Record) {
    LOGGER.log(record)
}

static LOGGER: Logger = Logger;

pub fn init() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::max());
}
