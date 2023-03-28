#![no_std]

trait BitMapTrait {
    fn is_alloc(&self, page: usize) -> bool {
        debug_assert!(page < self.page_count());
        self.get(page)
    }

    fn alloc(&mut self, page: usize) {
        debug_assert!(page < self.page_count());
        self.set(page, true);
    }

    fn free(&mut self, page: usize) {
        debug_assert!(page < self.page_count());
        self.set(page, false);
    }

    fn get_first_empty(&self, page_count: usize) -> Option<usize> {
        if page_count == 0 {
            return None;
        }
        let mut i = 0;
        while i < self.page_count() {
            let original_index = i;
            let mut count = page_count;
            loop {
                if self.get(i) {
                    break;
                } else {
                    count -= 1;
                    i += 1;
                    if count == 0 {
                        return Some(original_index);
                    }
                }
            }
            i += 1;
        }
        None
    }

    fn page_size(&self) -> usize;

    fn page_count(&self) -> usize;

    fn get(&self, index: usize) -> bool;

    fn set(&mut self, index: usize, val: bool);
}

/// `EXPONENT` uses the formula '2^`EXPONENT`' in order to determine page size.
/// `COUNT` uses the formula 8*`COUNT` to determine the amount of pages.
pub struct BitMap<const EXPONENT: usize, const COUNT: usize> {
    map: [u8; COUNT],
}

/// `EXPONENT` uses the formula '2^`EXPONENT`' in order to determine page size.
pub struct BitMapPtr<const EXPONENT: usize> {
    base: *mut u8,
    len: usize,
}

impl<const EXPONENT: usize, const COUNT: usize> BitMapTrait for BitMap<EXPONENT, COUNT> {
    fn page_size(&self) -> usize {
        Self::PAGE_SIZE
    }

    fn page_count(&self) -> usize {
        Self::PAGE_COUNT
    }

    fn get(&self, index: usize) -> bool {
        let i = index / 8;
        let j = index % 8;
        let b = 1 << j;
        let v = self.map[i];
        v & b != 0
    }

    fn set(&mut self, index: usize, val: bool) {
        let i = index / 8;
        let j = index % 8;
        let b = 1 << j;
        let mut v = self.map[i];
        v &= !b;
        v |= b * val as u8;
        self.map[i] = v;
    }
}

impl<const EXPONENT: usize> BitMapTrait for BitMapPtr<EXPONENT> {
    fn page_size(&self) -> usize {
        Self::PAGE_SIZE
    }

    fn page_count(&self) -> usize {
        self.len
    }

    fn get(&self, index: usize) -> bool {
        let i = index / 8;
        let j = index % 8;
        let b = 1 << j;
        let v = unsafe { *self.base.add(i) };
        v & b != 0
    }

    fn set(&mut self, index: usize, val: bool) {
        let i = index / 8;
        let j = index % 8;
        let b = 1 << j;
        let mut v = unsafe { *self.base.add(i) };
        v &= !b;
        v |= b * val as u8;
        unsafe { *self.base.add(i) = v };
    }
}

macro_rules! impl_func_wrapper {
    () => {
        pub fn is_alloc(&self, page: usize) -> bool {
            BitMapTrait::is_alloc(self, page)
        }

        pub fn alloc(&mut self, page: usize) {
            BitMapTrait::alloc(self, page)
        }

        pub fn free(&mut self, page: usize) {
            BitMapTrait::free(self, page)
        }

        pub fn get_first_empty(&self, page_count: usize) -> Option<usize> {
            BitMapTrait::get_first_empty(self, page_count)
        }

        pub fn page_size(&self) -> usize {
            BitMapTrait::page_size(self)
        }

        pub fn page_count(&self) -> usize {
            BitMapTrait::page_count(self)
        }
    };
}

impl<const EXPONENT: usize, const COUNT: usize> BitMap<EXPONENT, COUNT> {
    const PAGE_SIZE: usize = 1 << EXPONENT;
    const PAGE_COUNT: usize = COUNT * 8;

    pub const fn new() -> Self {
        Self { map: [0; COUNT] }
    }

    impl_func_wrapper!();
}

impl<const EXPONENT: usize> BitMapPtr<EXPONENT> {
    const PAGE_SIZE: usize = 1 << EXPONENT;

    pub const fn new(base: *mut u8, len: usize) -> Self {
        Self { base, len }
    }

    impl_func_wrapper!();
}
