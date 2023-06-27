#![no_std]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(int_roundings)]
#![feature(const_trait_impl)]
#![feature(pointer_is_aligned)]
#![feature(error_in_core)]
#![feature(ptr_metadata)]

#[macro_use]
extern crate alloc;

pub mod intrustive;

pub mod bitmap;
pub mod buddy;
pub mod freelist;
