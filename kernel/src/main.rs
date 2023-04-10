#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(trait_alias)]
#![feature(const_maybe_uninit_zeroed)]
#![feature(int_roundings)]
#![feature(const_slice_from_raw_parts_mut)]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]

#[macro_use]
extern crate static_assertions;

#[macro_use]
extern crate cfg_if;

#[macro_use]
extern crate log;

#[macro_use]
extern crate ctor;

#[macro_use]
extern crate proc_bitfield;

#[macro_use]
extern crate bitset;

#[macro_use]
extern crate alloc;

mod arch;
mod boot;
mod drivers;
mod logging;
mod mm;
mod panic;
mod util;

fn main() -> ! {
    info!("entered kernel main...");
    //drivers::vga::puts("hello vga!");
    //util::irq_en();
    loop {
        util::halt();
    }
}
