use core::slice;
use core::str;

pub unsafe fn syscall(syscall: u64, param0: u64, param1: u64) -> u64 {
    match syscall {
        0x10 => kprint(param0 as *const _, param1 as usize),
        _ => panic!("invalid syscall '{syscall}'"),
    }
}

unsafe fn kprint(ptr: *const u8, len: usize) -> u64 {
    let str = str::from_utf8_unchecked(slice::from_raw_parts(ptr, len));
    info!("kprint: {str}");
    0
}
