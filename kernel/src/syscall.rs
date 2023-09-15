use crate::mm::pmm;
use crate::mm::vmm::Flags;
use crate::mm::vmm::MapTy;
use crate::process::thread;
use crate::process::thread::Thread;
use ::syscall as sc;
use core::slice;
use core::str;

pub unsafe fn syscall(syscall: u64, param0: u64, param1: u64) -> u64 {
    let mut cur_thread = thread::cur_thread().get_locked();
    match syscall {
        sc::SYSCALL_KPRINT => kprint(param0 as *const _, param1 as usize),
        sc::SYSCALL_FREE => free(param0 as *const u8, param1 as usize),
        sc::SYSCALL_MALLOC => malloc(&mut cur_thread, param0 as usize),
        _ => panic!("invalid syscall '{syscall}'"),
    }
}

unsafe fn kprint(ptr: *const u8, len: usize) -> u64 {
    let str = str::from_utf8_unchecked(slice::from_raw_parts(ptr, len));
    info!("kprint: {str}");
    0
}

unsafe fn free(_ptr: *const u8, _len: usize) -> u64 {
    0
}

unsafe fn malloc(cur_thread: &mut Thread, len: usize) -> u64 {
    let mut proc = cur_thread.get_proc().get_locked();
    let pages = pages!(len);
    proc.vmm
        .map(
            None,
            pages,
            Flags::RW,
            MapTy::Phys {
                adr: pmm::alloc_pages(pages).phys(),
            },
        )
        .unwrap()
        .adr()
}
