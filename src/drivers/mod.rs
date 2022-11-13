#[cfg(target_arch = "x86_64")]
pub mod com;
pub mod ps2;

pub fn init() {
    com::init();
}
