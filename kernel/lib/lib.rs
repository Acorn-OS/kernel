#![no_std]
#![no_main]
#![feature(int_roundings)]
#![feature(alloc_error_handler)]
#![feature(const_maybe_uninit_zeroed)]
#![feature(generic_arg_infer)]

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate static_assertions;

#[macro_use]
extern crate log;

#[macro_use]
extern crate proc_bitfield;

pub mod allocators;
pub mod arch;
pub mod klog;
pub mod mm;
