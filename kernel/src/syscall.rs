use crate::arch::vm;
use crate::mm::pmm;
use crate::mm::vmm::Flags;
use crate::process::thread::{self, Thread};
use ::syscall as sc;
use core::slice;
use core::str;

pub unsafe fn syscall(syscall: u64, param0: u64, param1: u64) -> u64 {
    let cur_thread = thread::cur_thread().as_mut();
    debug_assert!(!cur_thread.get().is_kernel_thread());
    match syscall {
        sc::SYSCALL_KPRINT => kprint(param0 as *const _, param1 as usize),
        sc::SYSCALL_FREE => free(param0 as *const u8, param1 as usize),
        sc::SYSCALL_MALLOC => malloc(cur_thread, param0 as usize),
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
    let mut proc = cur_thread.get().get_proc().as_ref().lock();
    let pages = pages!(len);
    let adr = proc.vmm.map(
        None,
        pages,
        Flags::Phys {
            flags: vm::Flags::PRESENT | vm::Flags::RW | vm::Flags::XD,
            phys: pmm::alloc_pages(pages).phys(),
        },
    );
    adr.adr()
}
