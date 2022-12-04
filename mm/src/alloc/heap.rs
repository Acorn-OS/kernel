mod alloc;
mod free;

use core::{
    alloc::{GlobalAlloc, Layout},
    mem::size_of,
    ptr,
    slice::from_raw_parts_mut,
};

const BLOCK_SIZE: usize = vm::KERNEL_PG_SIZE;

#[repr(C)]
#[derive(Clone, Copy)]
struct BlockDesc(u8);

const_assert!(size_of::<BlockDesc>() == 1);

impl BlockDesc {
    fn new_free() -> Self {
        Self(0)
    }

    fn new_allocated() -> Self {
        Self(1)
    }

    fn is_free(&self) -> bool {
        self.0 == 0
    }
}

struct Heap {
    block_offset: usize,
    block_desc_offset: usize,
    block_count: usize,
}

impl Heap {
    unsafe fn init(&mut self) {
        assert!(vm::kwm_start() & vm::KERNEL_PG_MASK == 0);
        self.block_count = 4096 * 4;
        let block_desc_alloc = self.block_count;
        let block_alloc = self.block_count * BLOCK_SIZE;
        assert!(block_alloc + block_desc_alloc < vm::kwm_len());
        self.block_desc_offset = vm::kwm_start();
        self.block_offset = vm::kwm_start() + self.block_count;
        for block in self.get_block_desc() {
            *block = BlockDesc::new_free();
        }
    }

    fn get_block_desc(&self) -> &'static mut [BlockDesc] {
        unsafe { from_raw_parts_mut(self.block_desc_offset as *mut BlockDesc, self.block_count) }
    }

    fn ptr_to_index<T>(&self, ptr: *const T) -> usize {
        let ptr = ptr as usize;
        assert!(
            ptr >= self.block_offset && ptr < self.block_offset + self.block_count * BLOCK_SIZE
        );
        let ptr = ptr - self.block_offset;
        ptr / BLOCK_SIZE
    }

    fn index_to_ptr<T>(&self, index: usize) -> *mut T {
        assert!(index < self.block_count);
        let ptr = self.block_offset + index * BLOCK_SIZE;
        ptr as *mut T
    }

    fn find_empty_consecutive(&self, count: usize) -> Option<usize> {
        let mut i = 0;
        let empty_blocks = self.get_block_desc();
        while i < self.block_count {
            let mut found = 0;
            let start_index = i;
            while i < self.block_count && found < count && empty_blocks[i].is_free() {
                found += 1;
                i += 1;
            }
            if found >= count {
                return Some(start_index);
            }
            i += (found == 0) as usize;
        }
        None
    }

    fn get_block_count_from_size(&self, size: usize) -> usize {
        (size / BLOCK_SIZE) + (size % BLOCK_SIZE != 0) as usize
    }
}

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let allocated_blocks = self.get_block_count_from_size(layout.size());
        let index = self.find_empty_consecutive(allocated_blocks);
        if let Some(index) = index {
            let blocks = self.get_block_desc();
            for i in 0..allocated_blocks {
                blocks[i + index] = BlockDesc::new_allocated();
            }
            self.index_to_ptr(index)
        } else {
            error!("Unable to allocate enough empty consecutive blocks.");
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let index = self.ptr_to_index(ptr);
        let count = self.get_block_count_from_size(layout.size());
        let blocks = self.get_block_desc();
        for i in 0..count {
            let index = index + i;
            blocks[index] = BlockDesc::new_free();
        }
    }
}

#[global_allocator]
static mut ALLOCATOR: Heap = Heap {
    block_offset: 0,
    block_desc_offset: 0,
    block_count: 0,
};

pub fn init() {
    once! {
        unsafe { ALLOCATOR.init() };
    }
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    error!("allocation error: layout {layout:?}");
    loop {
        crate::util::halt()
    }
}
