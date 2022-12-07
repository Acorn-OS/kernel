pub extern crate log;

/// Should only be called once.
pub unsafe fn init_logging(logger: &'static impl log::Log) {
    log::set_logger(logger).expect("failed to set logger");
    log::set_max_level(log::LevelFilter::max());
}
