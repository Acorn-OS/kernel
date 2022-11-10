use crate::{drivers, fb};

struct KLog;

impl log::Log for KLog {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
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
    log::set_logger(&KLOG).expect("failed to set logger");
    log::set_max_level(log::LevelFilter::max());
}
