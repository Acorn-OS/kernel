use core::slice;
use core::str;

pub const SYSCALL_KPRINT: u64 = 0x0;
pub const SYSCALL_MALLOC: u64 = 0x1;
pub const SYSCALL_FREE: u64 = 0x2;

pub unsafe fn syscall(syscall: u64, param0: u64, param1: u64) -> u64 {
    match syscall {
        SYSCALL_KPRINT => kprint(param0 as *const _, param1 as usize),
        SYSCALL_FREE => free(param0 as *const u8, param1 as usize),
        SYSCALL_MALLOC => malloc(param0 as usize),
        _ => panic!("invalid syscall '{syscall}'"),
    }
}

unsafe fn kprint(ptr: *const u8, len: usize) -> u64 {
    let str = str::from_utf8_unchecked(slice::from_raw_parts(ptr, len));
    info!("kprint: {str}");
    0
}

unsafe fn free(_ptr: *const u8, _len: usize) -> u64 {
    todo!()
}

unsafe fn malloc(_len: usize) -> u64 {
    todo!()
}
