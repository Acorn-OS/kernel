#![no_std]

use core::arch::asm;

pub const SYSCALL_KPRINT: u64 = 0x0;
pub const SYSCALL_MALLOC: u64 = 0x1;
pub const SYSCALL_FREE: u64 = 0x2;

#[inline]
fn syscall(syscall: u64, param0: u64, param1: u64) -> u64 {
    let mut out: u64;
    unsafe {
        asm!(
            "syscall",
            in("rdi") syscall,
            in("rsi") param0,
            in("rdx") param1,
            out("rax") out,
        );
    }
    out
}

pub fn kprint(msg: impl AsRef<str>) {
    syscall(
        SYSCALL_KPRINT,
        msg.as_ref().as_ptr() as u64,
        msg.as_ref().len() as u64,
    );
}

pub fn malloc(count: usize) -> *mut u8 {
    syscall(SYSCALL_MALLOC, count as u64, 0) as *mut _
}

pub fn free(ptr: *mut u8, len: usize) {
    syscall(SYSCALL_FREE, ptr as u64, len as u64);
}
