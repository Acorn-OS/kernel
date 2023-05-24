#![no_std]

use core::arch::asm;
use kernel::syscall as sc;

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
        sc::SYSCALL_KPRINT,
        msg.as_ref().as_ptr() as u64,
        msg.as_ref().len() as u64,
    );
}

pub fn malloc(count: usize) -> *mut u8 {
    syscall(sc::SYSCALL_MALLOC, count as u64, 0) as *mut _
}

pub fn free(ptr: *mut u8, len: usize) {
    syscall(sc::SYSCALL_FREE, ptr as u64, len as u64);
}
