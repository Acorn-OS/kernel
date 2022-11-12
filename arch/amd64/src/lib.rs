#![no_std]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[allow(unused_imports)]
#[macro_use]
extern crate static_assertions;

#[allow(unused_imports)]
#[macro_use]
extern crate util;

pub mod chipset;
pub mod cpu;

mod hal;
