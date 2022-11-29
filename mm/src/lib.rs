#![no_std]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate static_assertions;

#[macro_use]
extern crate util;

pub mod malloc;
pub mod mmap;
pub mod paging;
pub mod vm;
