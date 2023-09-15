#![allow(unused)]

macro_rules! is_aligned {
    ($expr:expr, $align:expr) => {{
        ($expr) & ($align - 1) == 0
    }};
}

macro_rules! align_floor {
    ($expr:expr, $align:expr) => {{
        ($expr).div_floor($align) * ($align)
    }};
}

macro_rules! align_ceil {
    ($expr:expr, $align:expr) => {{
        ($expr).div_ceil($align) * ($align)
    }};
}

macro_rules! pages {
    ($expr:expr) => {{
        let val = ($expr) as usize;
        (val).div_ceil($crate::mm::pmm::PAGE_SIZE)
    }};
}

#[derive(Clone, Copy)]
#[must_use]
#[repr(transparent)]
pub struct Flags(u32);

impl Flags {
    pub const NONE: Flags = Flags(0);
    pub const PRESENT: Flags = Flags(1 << 0);
    pub const RW: Flags = Flags(1 << 1);
    pub const USER: Flags = Flags(1 << 2);
    const PS: Flags = Flags(1 << 7);
    const RESV: Flags = Flags(1 << 9);

    pub const SIZE_LARGE: Flags = Flags(1 << 16);
    pub const SIZE_MEDIUM: Flags = Flags(1 << 17);
    pub const XD: Flags = Flags(1 << 18);

    pub const fn merge(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn has(self, flags: Flags) -> bool {
        self.0 & flags.0 == flags.0
    }
}

impl core::ops::BitOr for Flags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.merge(rhs)
    }
}

macro_rules! bit_flags {
    (
        pub struct $ident:ident($inner_ty:ty);
        $($f_ident:ident = $expr:expr;)*
    ) => {
        #[derive(Clone, Copy)]
        #[must_use]
        #[repr(transparent)]
        pub struct $ident($inner_ty);

        impl $ident {
            pub const NONE: $ident = $ident(0);
            $(
                pub const $f_ident: $ident = $ident(1 << ($expr));
            )*

            pub const fn merge(self, other: Self) -> Self {
                Self(self.0 | other.0)
            }

            pub const fn has(self, flags: Self) -> bool {
                self.0 & flags.0 == flags.0
            }

            pub const fn has_any(self, flags: Self) -> bool {
                self.0 & flags.0 != 0
            }
        }

        impl core::ops::BitOr for $ident {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self::Output {
                self.merge(rhs)
            }
        }

        impl core::ops::BitOrAssign for $ident {
            fn bitor_assign(&mut self, rhs: Self) {
                *self = *self | rhs;
            }
        }
    };
}
