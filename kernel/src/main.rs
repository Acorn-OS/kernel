#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(trait_alias)]
#![feature(const_maybe_uninit_zeroed)]
#![feature(int_roundings)]
#![feature(const_slice_from_raw_parts_mut)]

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

fn tmp() {
    use freelist::FreeList;

    info!("debugging freelists");

    let mut free_lists = FreeList::new();
    free_lists.push_region(0, 0x100).unwrap();
    info!("{free_lists:?}");
    let ptr = free_lists.alloc::<[u8; 0x20]>().expect("expected #0");
    info!("{free_lists:?}");
    free_lists.alloc::<[u8; 0x30]>().unwrap();
    info!("{free_lists:?}");
    free_lists.free(ptr).unwrap();
    info!("{free_lists:?}");

    info!("new scenario");

    let mut free_lists = FreeList::new();
    free_lists.push_region(0, 0x100).unwrap();
    info!("{free_lists:?}");
    free_lists.alloc::<[u8; 0x20]>().unwrap();
    info!("{free_lists:?}");
    let ptr = free_lists.alloc::<[u8; 0x30]>().expect("expected #1");
    info!("{free_lists:?}");
    free_lists.free(ptr).unwrap();
    info!("{free_lists:?}");
}

fn main() -> ! {
    info!("entered kernel main...");
    tmp();
    //drivers::vga::puts("hello vga!");
    util::irq_en();
    loop {
        util::halt();
    }
}
