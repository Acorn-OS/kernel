#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(trait_alias)]
#![feature(const_maybe_uninit_zeroed)]
#![feature(int_roundings)]
#![feature(const_slice_from_raw_parts_mut)]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(error_in_core)]
#![feature(ptr_metadata)]
#![feature(ptr_from_ref)]
#![feature(if_let_guard)]
#![feature(inline_const)]
#![feature(strict_provenance)]

#[macro_use]
extern crate static_assertions;

#[macro_use]
extern crate cfg_if;

#[macro_use]
extern crate log;

//#[macro_use]
extern crate ctor;

#[macro_use]
extern crate proc_bitfield;

#[macro_use]
extern crate bitset;

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate memoffset;

#[macro_use]
mod macros;

mod arch;
mod boot;
mod drivers;
mod fs;
mod kernel_elf;
mod logging;
mod mm;
mod panic;
mod process;
mod symbols;
mod syscall;
mod util;

use crate::fs::{initrd, Vfs};
use alloc::string::String;
use boot::BootInfo;
use core::ffi::CStr;

fn main(boot_info: BootInfo) -> ! {
    info!("entered kernel main...");
    info!("loaded modules count: {}", boot_info.modules.module_count);
    info!("loaded modules:");
    for i in 0..boot_info.modules.module_count as usize {
        let path = unsafe {
            let ptr = boot_info.modules.modules.as_ptr().add(i);
            CStr::from_ptr((*ptr).path.as_ptr().unwrap())
        };
        info!("    {}", String::from_utf8_lossy(path.to_bytes()));
    }
    let mut initrd = None;
    for i in 0..boot_info.modules.module_count as usize {
        let ptr = unsafe { boot_info.modules.modules.as_ptr().add(i) };
        let path = unsafe { CStr::from_ptr((*ptr).path.as_ptr().unwrap()) };
        if path.to_bytes() == "/modules/initrd".as_bytes() {
            let mut vfs = unsafe {
                initrd::InitrdFs::from_raw((*ptr).base.as_ptr().unwrap(), (*ptr).length as usize)
            };
            let metadata = vfs.ls("").unwrap();
            info!("found {} files in initrd", metadata.len());
            info!("initrd:");
            for metadata in metadata {
                info!("    {} {}", metadata.name, metadata.size);
            }
            initrd = Some(vfs);
        }
    }
    process::run(initrd.expect("failed to locate initrd"))
}
