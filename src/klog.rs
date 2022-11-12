use crate::{drivers, fb};
use util::logging::log::{Log, Metadata, Record};

struct KLog;

impl Log for KLog {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        ::core::fmt::write(
            unsafe { &mut *(self as *const Self as *mut Self) },
            format_args!("{}\n\r", record.args()),
        )
        .expect("failed to format in logging");
    }

    fn flush(&self) {}
}

impl core::fmt::Write for KLog {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        drivers::com::puts(s);
        fb::puts(s);
        Ok(())
    }
}

static KLOG: KLog = KLog;

pub fn init() {
    unsafe { util::logging::init_logging(&KLOG) };
}
