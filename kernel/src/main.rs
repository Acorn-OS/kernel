#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(const_maybe_uninit_zeroed)]

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

mod arch;
mod boot;
mod logging;
mod panic;
mod util;

fn main() -> ! {
    info!("Acorn OS");
    loop {}
}
