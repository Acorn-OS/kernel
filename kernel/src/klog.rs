use util::logging::log::{Log, Metadata, Record};

struct KLog {
    serial_logging: fn(&str),
    fb_logging: fn(&str),
}

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
        (self.serial_logging)(s);
        (self.fb_logging)(s);
        Ok(())
    }
}

fn dummy_logging(_: &str) {}

static mut KLOG: KLog = KLog {
    serial_logging: dummy_logging,
    fb_logging: dummy_logging,
};

pub unsafe fn configure(serial_logging: fn(&str), fb_logging: fn(&str)) {
    KLOG.serial_logging = serial_logging;
    KLOG.fb_logging = fb_logging;
}

pub unsafe fn init() {
    unsafe { util::logging::init_logging(&KLOG) };
}
