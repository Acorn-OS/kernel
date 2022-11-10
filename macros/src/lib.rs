#![no_std]

extern crate self as proc_macro;

pub mod __private {
    pub use spin;
}

pub use defs::*;
