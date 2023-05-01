#![no_std]
use core::mem::size_of;

pub trait BitSet {
    fn set(&mut self, index: usize);

    fn clear(&mut self, index: usize);

    fn toggle(&mut self, index: usize, val: bool);
}

macro_rules! impl_primitive {
    ($($ty:ty),*) => {
        $(
            impl BitSet for $ty {
                fn set(&mut self, index: usize) {
                    debug_assert!(index < size_of::<Self>() * 8, "index out of bounds");
                    let b = 1 << index;
                    *self = *self | b;
                }

                fn clear(&mut self, index: usize) {
                    debug_assert!(index < size_of::<Self>() * 8, "index out of bounds");
                    let b = 1 << index;
                    *self = *self & !b;
                }

                fn toggle(&mut self, index: usize, val: bool) {
                    debug_assert!(index < size_of::<Self>() * 8, "index out of bounds");
                    let b = 1 << index;
                    *self &= !b;
                    *self |= b * val as Self;
                }
            }
        )*
    };
}
impl_primitive!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);

pub fn set(v: &mut impl BitSet, index: usize) {
    v.set(index)
}

pub fn clear(v: &mut impl BitSet, index: usize) {
    v.clear(index)
}

pub fn toggle(v: &mut impl BitSet, index: usize, val: bool) {
    v.toggle(index, val)
}

#[macro_export]
macro_rules! bset {
    ($val:expr, $bit:expr) => {
        $crate::set(&mut $val, $bit)
    };
}

#[macro_export]
macro_rules! bclear {
    ($val:expr, $bit:expr) => {
        $crate::clear(&mut $val, $bit)
    };
}

#[macro_export]
macro_rules! btoggle {
    ($val:expr, $bit:expr, $toggle:expr) => {
        $crate::toggle(&mut $val, $bit, $toggle)
    };
}
