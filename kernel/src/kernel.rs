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
mod init;
mod kernel_elf;
mod logging;
mod mm;
mod panic;
mod process;
mod scheduler;
mod symbols;
mod syscall;
mod util;
