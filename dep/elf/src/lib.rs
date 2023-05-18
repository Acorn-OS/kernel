#![no_std]
#![feature(error_in_core)]
#![feature(ptr_metadata)]
#![feature(pointer_is_aligned)]

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate static_assertions;

pub mod elf64;

pub mod dwarf;

#[allow(unused)]
mod elf32;

pub use elf64::Elf64;
