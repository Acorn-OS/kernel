#![no_std]
#![feature(allocator_api)]
#![feature(int_roundings)]

extern crate alloc;

use alloc::slice;
use core::alloc::{AllocError, Allocator, Layout};
use core::ptr::NonNull;
use spin::Mutex;

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

pub enum Error {
    OutOfSpace,
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::OutOfSpace => write!(f, "out of allocatable space"),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;

struct BitMapPtrAllocatorInner<const EXPONENT: usize> {
    bitmap: BitMapPtr<EXPONENT>,
    alloc_base: *mut u8,
}

impl<const EXPONENT: usize> BitMapPtrAllocatorInner<EXPONENT> {
    const fn new(bitmap_base: *mut u8, bitmap_len: usize, alloc_base: *mut u8) -> Self {
        Self {
            bitmap: BitMapPtr::new(bitmap_base, bitmap_len),
            alloc_base,
        }
    }
}

pub struct BitMapPtrAllocator<const EXPONENT: usize> {
    inner: Mutex<BitMapPtrAllocatorInner<EXPONENT>>,
}

unsafe impl<const EXPONENT: usize> Sync for BitMapPtrAllocator<EXPONENT> {}

impl<const EXPONENT: usize> BitMapPtrAllocator<EXPONENT> {
    const PAGE_SIZE: usize = BitMapPtr::<EXPONENT>::PAGE_SIZE;

    pub const unsafe fn new(bitmap_base: *mut u8, bitmap_len: usize, alloc_base: *mut u8) -> Self {
        Self {
            inner: Mutex::new(BitMapPtrAllocatorInner::new(
                bitmap_base,
                bitmap_len,
                alloc_base,
            )),
        }
    }

    pub unsafe fn init(&self, bitmap_base: *mut u8, bitmap_len: usize, alloc_base: *mut u8) {
        *self.inner.lock() = BitMapPtrAllocatorInner::new(bitmap_base, bitmap_len, alloc_base);
    }

    pub const fn page_size(&self) -> usize {
        Self::PAGE_SIZE
    }

    pub fn alloc_pages(&self, count: usize) -> Result<*mut u8> {
        let mut lock = self.inner.lock();
        if let Some(index) = lock.bitmap.get_first_empty(count) {
            for i in 0..count {
                lock.bitmap.alloc(index + i);
            }
            Ok(unsafe { lock.alloc_base.add(Self::PAGE_SIZE * index) })
        } else {
            Err(Error::OutOfSpace)
        }
    }

    pub fn free_pages(&self, ptr: *mut u8, count: usize) {
        let mut lock = self.inner.lock();
        debug_assert!(ptr as usize >= lock.alloc_base as usize);
        let index =
            (unsafe { ptr.sub(lock.alloc_base as usize) } as usize).div_floor(Self::PAGE_SIZE);
        for i in 0..count {
            lock.bitmap.free(index + i);
        }
    }

    fn alloc_layout(&self, layout: Layout) -> Result<*mut u8> {
        let pages = layout.size().div_ceil(Self::PAGE_SIZE);
        self.alloc_pages(pages)
    }

    fn free_layout(&self, ptr: *mut u8, layout: Layout) {
        let pages = layout.size().div_ceil(Self::PAGE_SIZE);
        self.free_pages(ptr, pages);
    }

    pub fn alloc<T>(&self) -> Result<*mut T> {
        let layout = Layout::new::<T>();
        self.alloc_layout(layout).map(|v| v as *mut T)
    }

    pub fn free<T>(&self, ptr: *mut T) {
        let layout = Layout::new::<T>();
        self.free_layout(ptr as *mut u8, layout)
    }

    pub fn alloc_bytes(&self, len: usize) -> Result<*mut u8> {
        self.alloc_layout(unsafe { Layout::from_size_align_unchecked(len, 1) })
    }

    pub fn free_bytes(&self, ptr: *mut u8, len: usize) {
        self.free_layout(ptr, unsafe { Layout::from_size_align_unchecked(len, 1) })
    }

    pub fn alloc_base(&self) -> *mut u8 {
        self.inner.lock().alloc_base
    }

    pub fn bitmap_base(&self) -> *mut u8 {
        self.inner.lock().bitmap.base
    }
}

unsafe impl<const EXPONENT: usize> Allocator for BitMapPtrAllocator<EXPONENT> {
    fn allocate(&self, layout: Layout) -> core::result::Result<NonNull<[u8]>, AllocError> {
        match self.alloc_layout(layout) {
            Ok(ptr) if ptr.is_null() => Err(AllocError),
            Ok(ptr) => {
                Ok(
                    unsafe {
                        NonNull::new_unchecked(slice::from_raw_parts_mut(ptr, layout.size()))
                    },
                )
            }
            Err(_) => Err(AllocError),
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.free_layout(ptr.as_ptr(), layout)
    }
}
