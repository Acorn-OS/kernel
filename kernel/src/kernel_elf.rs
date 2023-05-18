use crate::boot::BootInfo;
use elf::Elf64;

static mut KERNEL_ELF_BASE: (u64, usize) = (0, 0);

pub unsafe fn init(boot_info: &BootInfo) {
    let kernel_file = boot_info.file.kernel_file.get().unwrap();
    KERNEL_ELF_BASE = (
        kernel_file.base.as_ptr().unwrap() as u64,
        kernel_file.length as usize,
    )
}

pub fn elf() -> &'static Elf64 {
    unsafe { &*Elf64::from_raw_parts(KERNEL_ELF_BASE.0 as *const _, KERNEL_ELF_BASE.1) }
}
