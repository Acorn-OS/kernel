macro_rules! def_align {
    ($($ident:ident, $ty:ty);*;) => {
        $(
            pub fn $ident(v: $ty, align: $ty) -> $ty{
                assert!(align.is_power_of_two());
                v.div_ceil(align) << align
            }
        )*
    };
}

def_align! {
    align_u8, u8;
    align_u16, u16;
    align_u32, u32;
    align_u64, u64;
    align_u128, u128;
}
