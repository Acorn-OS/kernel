use core::mem::size_of;

use crate::arch::{self, padr, vadr};
use crate::mm::pmm;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PhysAdr(padr);

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct VirtAdr(vadr);

// check temporarily just to avoid potential bugs
const_assert!(size_of::<vadr>() == size_of::<u64>());
const_assert!(size_of::<padr>() == size_of::<u64>());

macro_rules! impl_adr {
    () => {
        pub const fn null() -> Self {
            Self::new(0)
        }

        pub const fn is_null(self) -> bool {
            self.0 == 0
        }

        #[inline]
        pub const fn align_ceil(self, align: usize) -> Self {
            assert!(align.is_power_of_two() && align != 0);
            let mask = align as padr - 1;
            Self((self.0 + mask) & !mask)
        }

        #[inline]
        pub const fn align_floor(self, align: usize) -> Self {
            assert!(align.is_power_of_two() && align != 0);
            let mask = align as padr - 1;
            Self(self.0 & !mask)
        }

        #[inline]
        pub const fn is_aligned(self, align: usize) -> bool {
            assert!(align.is_power_of_two() && align != 0);
            self.0 & (align as padr - 1) == 0
        }

        #[inline]
        pub const fn mask(self, mask: usize) -> Self {
            Self((self.0 as usize & mask) as _)
        }

        #[inline]
        pub const fn add(self, add: usize) -> Self {
            Self((self.0 as usize + add) as _)
        }

        #[inline]
        pub const fn sub(self, sub: usize) -> Self {
            Self((self.0 as usize - sub) as _)
        }
    };
}

impl PhysAdr {
    #[inline]
    pub const fn new(padr: padr) -> Self {
        Self(padr)
    }

    #[inline]
    pub const fn adr(self) -> padr {
        self.0
    }

    impl_adr!();
}

impl VirtAdr {
    #[inline]
    pub const fn new(vadr: vadr) -> Self {
        Self(vadr)
    }

    #[inline]
    pub const fn adr(self) -> vadr {
        self.0
    }

    pub fn to_phys(self, page_map: arch::vm::PageMapPtr) -> Option<PhysAdr> {
        if let Some(phys) = unsafe { arch::vm::get_page_entry(page_map, self) } {
            if unsafe { phys.as_ref().present() } {
                unsafe {
                    Some(PhysAdr::new(
                        phys.as_ref().adr().adr() + self.mask(pmm::PAGE_SIZE - 1).adr(),
                    ))
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    #[inline]
    pub const fn ptr(self) -> *mut u8 {
        self.0 as *mut _
    }

    impl_adr!();
}

impl core::fmt::Display for VirtAdr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("0x{:016x}", self.0))
    }
}

impl core::fmt::Display for PhysAdr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("0x{:016x}", self.0))
    }
}
