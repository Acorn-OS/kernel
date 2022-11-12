use core::{arch::asm, mem::size_of, ops::RangeInclusive};

#[derive(Clone, Copy)]
struct Entry(u64);

const_assert!(size_of::<Entry>() == 8);
const_assert!(size_of::<PageDirectory>() == 4096);
assert_eq_size!(usize, u64);

impl Entry {
    const ADR_MASK: u64 = 0x001F_FFFF_FFFF_F000;

    #[inline(always)]
    const fn new_empty() -> Self {
        Self(0)
    }

    #[inline(always)]
    fn new_rwp() -> Self {
        let mut new = Self::new_empty();
        new.set_rw_bit(true);
        new.set_present_bit(true);
        new
    }

    #[inline(always)]
    fn adr_space(&self) -> u64 {
        ((((self.0 & Self::ADR_MASK) as i64) << 11) >> 11) as u64
    }

    fn set_size_bit(&mut self, bit: bool) {
        self.0 = (self.0 & !(1 << 7)) | ((bit as u64) << 7);
    }

    #[inline(always)]
    fn set_adr_space(&mut self, adr: u64) {
        self.0 = (adr & Self::ADR_MASK) | (self.0 & !Self::ADR_MASK)
    }

    #[inline(always)]
    fn set_present_bit(&mut self, bit: bool) {
        self.0 = (self.0 & !1) | (bit as u64);
    }

    #[inline(always)]
    fn set_rw_bit(&mut self, bit: bool) {
        self.0 = (self.0 & !(1 << 1)) | ((bit as u64) << 1);
    }

    #[inline]
    fn enable_rwp(&mut self) {
        self.set_rw_bit(true);
        self.set_present_bit(true);
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.0 & 0b1 == 0
    }
}

#[repr(C, align(4096))]
struct PageDirectory {
    entries: [Entry; 512],
}

#[allow(dead_code)]
pub enum PageSize {
    /// 4KiB.
    Normal,
    /// 2MiB.
    Large,
    /// 1GiB.
    Huge,
}

impl PageSize {
    #[inline(always)]
    pub fn mask(&self) -> usize {
        self.value() - 1
    }

    #[inline(always)]
    pub fn log2(&self) -> usize {
        match self {
            PageSize::Normal => 12,
            PageSize::Large => 21,
            PageSize::Huge => 30,
        }
    }

    #[inline(always)]
    pub fn value(&self) -> usize {
        1 << self.log2()
    }
}

static mut PAGE_TABLE_BASE_ADR: usize = 0;

proc_macro::idef! {
    static ALLOCATOR = {}
    impl {
        #[inline(always)]
        pub unsafe fn install(&self) {
            let adr = self.get_base() as *const _ as u64;
            asm!(
                "mov cr3, rax",
                in("rax") adr,
                options(nostack)
            );
        }

        #[inline(always)]
        pub fn map(&mut self, virt: RangeInclusive<usize>, phys: usize, size: PageSize) {
            let mask = size.mask();
            let mut phys = phys & !mask;
            let vstart = *virt.start() & !mask;
            let vend = *virt.end() & !mask;

            macro_rules! shft {
                ($val:literal) => {
                    (12 + (9 * $val))
                };
            }

            macro_rules! indexes {
                ($start:ident, $end:ident, $shift:literal) => {
                    let $start = (vstart >> shft!($shift)) & 0x1FF;
                    let $end = (vend >> shft!($shift)) & 0x1FF;
                };
            }

            macro_rules! get {
                ($parent:ident, $parent_index:expr) => {{
                    let e = unsafe { (*$parent).entries.get_unchecked_mut($parent_index) };
                    if e.is_empty() {
                        unsafe {
                            e.set_adr_space(alloc::alloc::alloc(alloc::alloc::Layout::new::<
                                PageDirectory,
                            >()) as u64)
                        }
                        e.enable_rwp();
                    }
                    e.adr_space() as *mut PageDirectory
                }};
            }

            indexes!(p3_s, p3_e, 3);
            indexes!(p2_s, p2_e, 2);
            indexes!(p1_s, p1_e, 1);
            indexes!(p0_s, p0_e, 0);

            match size {
                PageSize::Normal => {
                    let start = p0_s | (p1_s << 9) | (p2_s << 18) | (p3_s << 27);
                    let end = p0_e | (p1_e << 9) | (p2_e << 18) | (p3_e << 27);

                    for i in start..=end {
                        let p0_i = i & 0x1FF;
                        let p1_i = (i >> 9) & 0x1FF;
                        let p2_i = (i >> 18) & 0x1FF;
                        let p3_i = (i >> 27) & 0x1FF;

                        let p3 = self.get_base();
                        let p2 = get!(p3, p3_i);
                        let p1 = get!(p2, p2_i);
                        let p0 = get!(p1, p1_i);

                        let entry = unsafe { (*p0).entries.get_unchecked_mut(p0_i) };
                        entry.enable_rwp();
                        entry.set_adr_space((phys & !size.mask()) as u64);
                        phys += size.value();
                    }
                }
                PageSize::Large => {
                    let start = p1_s | (p2_s << 9) | (p3_s << 18);
                    let end = p1_e | (p2_e << 9) | (p3_e << 18);

                    for i in start..=end {
                        let p1_i = i & 0x1FF;
                        let p2_i = (i >> 9) & 0x1FF;
                        let p3_i = (i >> 18) & 0x1FF;

                        let p3 = self.get_base();
                        let p2 = get!(p3, p3_i);
                        let p1 = get!(p2, p2_i);

                        let entry = unsafe { (*p1).entries.get_unchecked_mut(p1_i) };
                        entry.enable_rwp();
                        entry.set_size_bit(true);
                        entry.set_adr_space((phys & !size.mask()) as u64);
                        phys += size.value();
                    }
                }
                PageSize::Huge => {
                    let start = p2_s | (p3_s << 9);
                    let end = p2_e | (p3_e << 9);

                    for i in start..=end {
                        let p2_i = i & 0x1FF;
                        let p3_i = (i >> 9) & 0x1FF;

                        let p3 = self.get_base();
                        let p2 = get!(p3, p3_i);

                        let entry = unsafe { (*p2).entries.get_unchecked_mut(p2_i) };
                        entry.enable_rwp();
                        entry.set_size_bit(true);
                        entry.set_adr_space((phys & !size.mask()) as u64);
                        phys += size.value();
                    }
                }
            }
        }

        #[inline(always)]
        pub fn unmap(&self) {
            unimplemented!()
        }

        #[inline(always)]
        pub fn is_mapped(&self) -> bool {
            unimplemented!()
        }

        fn get_base(&self) -> &'static mut PageDirectory {
            unsafe { &mut *(PAGE_TABLE_BASE_ADR as *mut PageDirectory) }
        }
    }
}

pub unsafe fn init(paging_base_adr: usize) {
    PAGE_TABLE_BASE_ADR = paging_base_adr;
}
