//! Shared IO operations of the x86 family.
#![allow(missing_docs, dead_code)]

use core::arch::asm;

#[inline(always)]
pub fn out8(port: u16, v: u8) {
    unsafe {
        asm! {
            "out dx, al",
            in("al") v,
            in("dx") port,
            options(nostack)
        };
    }
}

#[inline(always)]
pub fn out16(port: u16, v: u16) {
    unsafe {
        asm! {
            "out dx, ax",
            in("ax") v,
            in("dx") port,
            options(nostack)
        };
    }
}

#[inline(always)]
pub fn out32(port: u16, v: u32) {
    unsafe {
        asm! {
            "out dx, eax",
            in("eax") v,
            in("dx") port,
            options(nostack)
        };
    }
}

#[inline(always)]
pub fn in8(port: u16) -> u8 {
    let mut out: i8;
    unsafe {
        asm! {
            "in al, dx",
            out("al") out,
            in("dx") port,
            options(nostack)
        };
    }
    out as u8
}

#[inline(always)]
pub fn in16(port: u16) -> u16 {
    let mut out: u16;
    unsafe {
        asm! {
            "in al, dx",
            out("ax") out,
            in("dx") port,
            options(nostack)
        };
    }
    out
}

#[inline(always)]
pub fn in32(port: u16) -> u32 {
    let mut out: u32;
    unsafe {
        asm! {
            "in al, dx",
            out("eax") out,
            in("dx") port,
            options(nostack)
        };
    }
    out
}

pub fn wait() {
    use crate::util::delay;
    delay(512);
}
