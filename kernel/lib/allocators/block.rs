use core::{alloc::GlobalAlloc, ptr};

use spin::MutexGuard;

/// `SIZE` is in bytes.
pub struct BlockHeap {
    base: *mut u8,
    block_size_ln2: usize,
    block_count: usize,
}

pub struct SpinBlockHeap(spin::Mutex<BlockHeap>);

impl SpinBlockHeap {
    pub const fn new(heap: BlockHeap) -> Self {
        Self(spin::Mutex::new(heap))
    }

    pub fn lock(&self) -> MutexGuard<BlockHeap> {
        self.0.lock()
    }
}

unsafe impl Send for SpinBlockHeap {}
unsafe impl Sync for SpinBlockHeap {}

/// `f` should be a function which allocates bytes.
pub fn new(
    block_size_ln2: usize,
    blocks: usize,
    f: impl Fn(usize) -> *mut u8,
) -> Result<BlockHeap, ()> {
    let blocks = blocks / 8 + 1;
    let block_size = 1 << block_size_ln2;
    let alloc = blocks * block_size;
    let bytes = alloc + blocks;
    let ptr = f(bytes);
    unsafe { ptr.write_bytes(0, bytes) };
    if ptr != ptr::null_mut() {
        Ok(BlockHeap {
            base: ptr,
            block_size_ln2,
            block_count: blocks,
        })
    } else {
        debug!("unable to allocate {bytes} bytes of data for Heap with block size of {block_size} and block count of {blocks}");
        Err(())
    }
}

impl BlockHeap {
    /// `ptr` has to point to a valid address
    /// which has a `BlockHeap::RESVB` amount
    /// of reserved bytes for the allocator.
    /// # Safety
    /// if `ptr` points to an unreserved address
    /// or insufficient reserved space, it could
    /// mean allocating new data is dangerous.
    pub const unsafe fn from_parts(
        base: *mut u8,
        block_size_ln2: usize,
        block_count: usize,
    ) -> Self {
        Self {
            base: base,
            block_size_ln2,
            block_count,
        }
    }

    fn block_size(&self) -> usize {
        1 << self.block_size_ln2
    }

    fn mask(&self) -> usize {
        self.block_size() - 1
    }

    fn get_blocks_ptr(&self) -> *mut u8 {
        self.base
    }
    fn get_alloc_ptr(&self) -> *mut u8 {
        unsafe { self.base.add((self.block_count / 8) + 1) }
    }

    fn set_block(&self, index: usize, val: bool) {
        debug_assert!(
            index < self.block_count,
            "set -> {index} : {}",
            self.block_count
        );
        let index = index & self.mask();
        let blocks = self.get_blocks_ptr();
        let bit = 1 << (index % 8);
        let val = if val { bit } else { 0 };
        let index = index / 8;
        unsafe { *blocks.add(index) = (*blocks.add(index) & !bit) | val };
    }

    fn get_block(&self, index: usize) -> bool {
        debug_assert!(
            index < self.block_count,
            "get -> {index} : {}",
            self.block_count
        );
        let index = index & self.mask();
        let blocks = self.get_blocks_ptr();
        let bit = 1 << (index % 8);
        let index = index / 8;
        unsafe { *blocks.add(index) & bit != 0 }
    }

    fn get_free_space_index(&self, len: usize) -> Option<usize> {
        if len == 0 {
            return None;
        }
        let len = (len + self.block_size() - 1) >> self.block_size_ln2;
        let mut count = 0;
        for i in 0..self.block_count {
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
        let align_ofs = align % self.block_size();
        let align_pad = align - align_ofs;
        let size = bytes + align_pad;
        let block_index = self.get_free_space_index(size)?;
        let block_cnt = (size >> self.block_size_ln2) + 1;
        for block in 0..block_cnt {
            self.set_block(block_index + block, true);
        }
        Some((
            self.get_alloc_ptr()
                .add(block_index * self.block_size())
                .add(align_pad),
            block_cnt << self.block_size_ln2,
        ))
    }

    pub unsafe fn free(&self, adr: usize, len: usize) -> usize {
        debug_assert!(adr >= self.base as usize);
        let adr = adr - self.base as usize;
        let block_index = adr >> self.block_size_ln2;
        let block_cnt = (len >> self.block_size_ln2) + 1;
        for block in 0..block_cnt {
            self.set_block(block_index + block, false);
        }
        block_cnt
    }
}

unsafe impl GlobalAlloc for BlockHeap {
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
