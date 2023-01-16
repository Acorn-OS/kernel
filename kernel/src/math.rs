#![allow(dead_code)]

pub trait Alignable {
    fn align_ceil(&self, align: Self) -> Self;

    fn align_floor(&self, align: Self) -> Self;
}

macro_rules! impl_align {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Alignable for $ty {
                #[inline]
                fn align_ceil(&self, align: Self) -> Self {
                    self.div_ceil(align) * align
                }

                #[inline]
                fn align_floor(&self, align: Self) -> Self {
                    self.div_floor(align) * align
                }
            }
        )*
    };
}
impl_align!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);

#[inline(always)]
pub fn align_ceil<A: Alignable>(value: A, align: A) -> A {
    value.align_ceil(align)
}

#[inline(always)]
pub fn align_floor<A: Alignable>(value: A, align: A) -> A {
    value.align_floor(align)
}
