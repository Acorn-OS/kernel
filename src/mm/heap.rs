use core::{alloc::GlobalAlloc, ptr};

/// ```2^(16*BLOCK_SIZE)``` bytes of allocatable space.
pub struct Heap<const BLOCK_SIZE: usize> {
    base: *mut u8,
}

unsafe impl<const BLOCK_SIZE: usize> Send for Heap<BLOCK_SIZE> {}

impl<const BLOCK_SIZE: usize> Heap<BLOCK_SIZE> {
    const BLOCK_LG2: usize = { BLOCK_SIZE };
    const BLOCK_SIZE: usize = (1 << Self::BLOCK_LG2);
    const BLOCKS: usize = Self::BLOCK_SIZE;
    const MASK: usize = Self::BLOCK_SIZE - 1;

    pub unsafe fn new() -> Self {
        let ptr = super::wm::reserve_pages(Self::BLOCKS * Self::BLOCK_SIZE + 1);
        if ptr == ptr::null_mut() {
            panic!("Unable to create Heap: insufficient kernel work memory!")
        }
        Self { base: ptr }
    }

    fn get_blocks_ptr(&self) -> *mut u8 {
        self.base
    }
    fn get_alloc_ptr(&self) -> *mut u8 {
        unsafe { self.base.add(Self::BLOCK_SIZE) }
    }

    fn set_block(&self, index: usize, val: bool) {
        debug_assert!(index < Self::BLOCKS);
        let index = index & Self::MASK;
        let blocks = self.get_blocks_ptr();
        let bit = 1 << (index % 8);
        let val = bit * (val as u8);
        let index = index / 8;
        unsafe { *blocks.add(index) = (*blocks.add(index) & !bit) | val };
    }

    fn get_block(&self, index: usize) -> bool {
        debug_assert!(index < Self::BLOCKS);
        let index = index & Self::MASK;
        let blocks = self.get_blocks_ptr();
        let bit = 1 << (index % 8);
        let index = index / 8;
        unsafe { *blocks.add(index) & bit != 0 }
    }

    fn get_free_space_index(&self, len: usize) -> Option<usize> {
        if len == 0 {
            return None;
        }
        let len = (len + Self::BLOCK_SIZE - 1) >> Self::BLOCK_LG2;
        let mut count = 0;
        for i in 0..Self::BLOCKS {
            if !self.get_block(i) {
                count += 1;
                if count >= len {
                    return Some(i + 1 - count);
                }
            } else {
                count = 0;
            }
        }
        None
    }

    pub unsafe fn alloc(&self, bytes: usize, align: usize) -> Option<(*mut u8, usize)> {
        let align_ofs = align % Self::BLOCK_SIZE;
        let size = bytes + align_ofs;
        let block_index = self.get_free_space_index(size)?;
        let block_cnt = size >> Self::BLOCK_LG2;
        for block in 0..block_cnt {
            self.set_block(block_index + block, true);
        }
        let alloc_ptr = self.get_alloc_ptr();
        Some((
            alloc_ptr.add(block_index * Self::BLOCK_SIZE).add(align_ofs),
            block_cnt << Self::BLOCK_LG2,
        ))
    }

    pub unsafe fn free(&self, adr: usize, len: usize) -> usize {
        let block_index = adr >> Self::BLOCK_LG2;
        let block_cnt = (len + Self::BLOCKS - 1) >> Self::BLOCK_LG2;
        for block in 0..block_cnt {
            self.set_block(block_index + block, false);
        }
        block_cnt
    }
}

unsafe impl<const BLOCK_SIZE: usize> GlobalAlloc for Heap<BLOCK_SIZE> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let bytes = layout.size();
        let align = layout.align();
        let Some((ptr, size)) = self.alloc(bytes, align) else {
            return ptr::null_mut()
        };
        if size >= bytes {
            ptr
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let bytes = layout.size();
        self.free(ptr as usize, bytes);
    }
}
