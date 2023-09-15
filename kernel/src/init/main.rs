use crate::arch::interrupt;
use crate::boot::BootInfo;
use alloc::string::String;
use core::ffi::CStr;

pub unsafe extern "C" fn main() -> ! {
    info!("entered kernel main...");
    let boot_info = BootInfo::get();
    info!("loaded modules count: {}", boot_info.modules.module_count);
    info!("loaded modules:");
    for i in 0..boot_info.modules.module_count as usize {
        let path = unsafe {
            let ptr = boot_info.modules.modules.as_ptr().add(i);
            CStr::from_ptr((*ptr).path.as_ptr().unwrap())
        };
        info!("    {}", String::from_utf8_lossy(path.to_bytes()));
    }
    loop {
        debug!("main loop!");
        interrupt::halt();
    }
}
