use crate::arch::imp::gdt;
use crate::arch::imp::msr;
use alloc::slice;
use core::arch::global_asm;
use core::str;

pub fn init() {
    msr::set(msr::IA32_EFER, msr::get(msr::IA32_EFER) | 1);
    msr::set(
        msr::IA32_STAR,
        ((gdt::KERNEL_CODE_SELECTOR as u64) << 32) | ((gdt::USRSPC_CODE_32_SELECTOR as u64) << 48),
    );
    msr::set(msr::IA32_LSTAR, syscall_enter as u64);
    msr::set(msr::IA32_FMASK, 1 << 10 /* clears direction bit */);
}

extern "C" {
    fn syscall_enter();
}

global_asm!(include_str!("syscall.s"));

#[no_mangle]
unsafe extern "C" fn syscall_handler(syscall: u64, param0: u64, param1: u64) {
    match syscall {
        0x10 => kprint(param0 as *const _, param1 as usize),
        _ => panic!("invalid syscall '{syscall}'"),
    }
}

unsafe fn kprint(ptr: *const u8, len: usize) {
    let str = str::from_utf8_unchecked(slice::from_raw_parts(ptr, len));
    info!("kprint: {str}");
}
