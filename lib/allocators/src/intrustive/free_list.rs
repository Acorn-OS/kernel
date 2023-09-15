use core::alloc::Layout;
use core::fmt::{self, Display};
use core::ptr::null_mut;

#[derive(Debug)]
pub enum Error {
    NonBlockAlignedAllocation,
    InsufficientSpace,
    UnalignedRegionSize,
    UnalignedRegion,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonBlockAlignedAllocation => f.write_str("alignment is not block aligned"),
            Self::InsufficientSpace => f.write_str("out of space"),
            Self::UnalignedRegionSize => f.write_str("insufficient region size"),
            Self::UnalignedRegion => f.write_str("unaligned region"),
        }
    }
}

impl core::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Copy)]
struct IntrusiveNodePtr(*mut ());

impl IntrusiveNodePtr {
    const fn nullptr() -> Self {
        Self(null_mut())
    }

    const fn new(ptr: *mut ()) -> Self {
        Self(ptr)
    }

    fn next(self) -> IntrusiveNodePtr {
        unsafe { IntrusiveNodePtr((self.0 as *mut *mut ()).read()) }
    }

    fn set_next(self, next: IntrusiveNodePtr) {
        unsafe { (self.0 as *mut *mut ()).write(next.0) }
    }

    fn blocks(self) -> u64 {
        unsafe { (self.0 as *mut u64).add(1).read() }
    }

    fn set_blocks(self, blocks: u64) {
        unsafe { (self.0 as *mut u64).add(1).write(blocks) }
    }

    fn is_null(self) -> bool {
        self.0.is_null()
    }

    fn adr(self) -> u64 {
        self.0 as u64
    }
}

pub struct IntrusiveFreeList<const BLOCK_SIZE: usize> {
    head: IntrusiveNodePtr,
}

impl<const BLOCK_SIZE: usize> IntrusiveFreeList<BLOCK_SIZE> {
    const _ASSERT: () = {
        assert!(BLOCK_SIZE.is_power_of_two());
        assert!(BLOCK_SIZE >= 16);
    };

    pub const fn new() -> Self {
        Self {
            head: IntrusiveNodePtr::nullptr(),
        }
    }

    const fn block_size() -> u64 {
        BLOCK_SIZE as u64
    }

    #[inline]
    fn block_cnt(byte_len: u64) -> u64 {
        (byte_len as u64 + (Self::block_size() - 1)) / Self::block_size()
    }

    /// push a region aligned by 8 bytes with and size has to be a multiple of 16, due to
    /// 16 bytes being used to describe the next entry and the size of the current entry.
    pub unsafe fn push_region_checked(&mut self, ptr: *mut u8, len: usize) -> Result<()> {
        if ptr as u64 % Self::block_size() != 0 {
            Err(Error::UnalignedRegion)
        } else if len as u64 % Self::block_size() != 0 {
            Err(Error::UnalignedRegionSize)
        } else {
            self.push_region_unchecked(ptr, len);
            Ok(())
        }
    }

    /// push a region aligned by 8 bytes with a size of at least 16, due to the first
    /// 16 bytes being used to describe the next entry and the size of the current entry.
    pub unsafe fn push_region_unchecked(&mut self, ptr: *mut u8, len: usize) {
        debug_assert!(ptr as u64 % Self::block_size() == 0, "ptr: {ptr:?}");
        debug_assert!(len as u64 % Self::block_size() == 0, "len: {len}");
        let node = IntrusiveNodePtr::new(ptr as *mut _);
        let blocks = Self::block_cnt(len as u64);
        node.set_blocks(blocks);
        node.set_next(self.head);
        self.head = node;
    }

    pub fn alloc_layout(&mut self, layout: Layout) -> Result<*mut u8> {
        if layout.align() > BLOCK_SIZE as usize {
            return Err(Error::NonBlockAlignedAllocation);
        }
        let mut cur = self.head;
        let mut prev = IntrusiveNodePtr::nullptr();
        while !cur.is_null() {
            let adr = cur.adr();
            let block_cnt = cur.blocks();
            let alloc_block_cnt = Self::block_cnt((layout.size() as u64).max(1));
            if alloc_block_cnt <= block_cnt {
                let remaining_blocks = block_cnt - alloc_block_cnt;
                if remaining_blocks > 0 {
                    let node = IntrusiveNodePtr::new(
                        (cur.adr() + alloc_block_cnt * Self::block_size()) as *mut _,
                    );
                    node.set_next(cur.next());
                    node.set_blocks(remaining_blocks);
                    if !prev.is_null() {
                        prev.set_next(node);
                    } else {
                        self.head = node;
                    }
                } else {
                    if !prev.is_null() {
                        prev.set_next(cur.next())
                    } else {
                        self.head = cur.next();
                    }
                }
                return Ok(adr as *mut u8);
            }
            prev = cur;
            cur = cur.next();
        }
        Err(Error::InsufficientSpace)
    }

    #[inline]
    pub fn alloc<T>(&mut self) -> Result<*mut T> {
        self.alloc_layout(Layout::new::<T>()).map(|v| v as *mut _)
    }

    #[inline]
    pub fn alloc_bytes(&mut self, bytes: usize) -> Result<*mut u8> {
        unsafe { self.alloc_layout(Layout::from_size_align_unchecked(bytes, 1)) }
    }

    #[inline]
    pub fn free_layout(&mut self, ptr: *mut u8, layout: Layout) {
        let block_cnt = Self::block_cnt(layout.size().max(1) as u64);
        let adr = ptr as u64;
        let end_adr = adr + block_cnt * Self::block_size();
        let node = IntrusiveNodePtr::new(ptr as *mut _);
        let mut cur = self.head;
        let mut prev = IntrusiveNodePtr::nullptr();
        while !cur.is_null() {
            let cur_adr = cur.adr();
            let cur_end_adr = cur.adr() + cur.blocks() * Self::block_size();
            if cur_adr == end_adr {
                node.set_blocks(block_cnt + cur.blocks());
                node.set_next(cur.next());
                if !prev.is_null() {
                    prev.set_next(node);
                } else {
                    self.head.set_next(node);
                }
                return;
            } else if cur_end_adr == cur_adr {
                cur.set_blocks(cur.blocks() + block_cnt);
                return;
            }
            prev = cur;
            cur = cur.next();
        }
        unsafe { self.push_region_unchecked(ptr, (block_cnt * Self::block_size()) as usize) }
    }

    #[inline]
    pub fn free<T>(&mut self, ptr: *mut T) {
        self.free_layout(ptr as *mut u8, Layout::new::<T>())
    }

    #[inline]
    pub fn free_bytes(&mut self, ptr: *mut u8, len: usize) {
        self.free_layout(ptr, unsafe { Layout::from_size_align_unchecked(len, 1) })
    }
}
