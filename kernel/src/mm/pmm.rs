use crate::boot::limine;
use core::ptr::null_mut;
use core::slice;

pub const PAGE_EXP: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_EXP;
const MIN_PAGE_COUNT: usize = 4096;

type BitMap = bitmap::BitMapPtr<PAGE_EXP>;

static mut PHYS_MEM_BASE_ADR: u64 = 0;
static mut BITMAP: BitMap = BitMap::new(null_mut(), 0);


/// Allocate `count` amount of pages in physical memory.
pub unsafe fn alloc_pages(count: usize) -> *mut u8 {
    if let Some(index) = BITMAP.get_first_empty(count) {
        for i in 0..count {
            BITMAP.alloc(i + index);
        }
        (PHYS_MEM_BASE_ADR as usize + index * BITMAP.page_size()) as *mut u8
    } else {
        panic!("out of physical memory")
    }
}

/// Allocate `count` amount of zeroed pages in physical memory.
pub unsafe fn alloc_pages_zeroed(count: usize) -> *mut u8 {
    let ptr = alloc_pages(count);
    ptr.write_bytes(0, count * PAGE_SIZE);
    ptr
}

/// Deallocates `count` pages at physical address `ptr`.
pub unsafe fn dealloc_pages(ptr: *const u8, count: usize) {
    debug_assert!(
        ptr as u64 >= PHYS_MEM_BASE_ADR
            && (ptr as u64) < PHYS_MEM_BASE_ADR + BITMAP.page_count() as u64
    );
    let adr = ptr as u64;
    let page = (adr - PHYS_MEM_BASE_ADR) / PAGE_SIZE as u64;
    for i in 0..count {
        BITMAP.free(page as usize + i);
    }
}

pub unsafe fn init() {
    fn align_floor(val: usize, align: usize) -> usize {
        val.div_floor(align) * align
    }

    fn align_ceil(val: usize, align: usize) -> usize {
        val.div_ceil(align) * align
    }

    let mmap = limine::mmap();
    let count = mmap.entry_count;
    let entries = *mmap.entries;
    let entries = slice::from_raw_parts_mut(entries, count as usize);
    let mut found = false;
    for (index, entry) in entries.iter().enumerate() {
        if entry.ty != limine::MMapEntry::USABLE {
            continue;
        }
        let mut base = entry.base as usize;
        let mut len = entry.len as usize;

        let bitmap_base = base;
        let bitmap_len = (len / PAGE_SIZE) / 8;

        len = align_floor(len - bitmap_len, PAGE_SIZE);
        base = align_ceil(base + bitmap_len, PAGE_SIZE);

        if len / PAGE_SIZE < MIN_PAGE_COUNT {
            continue;
        }
        let bitmap_len = (len / PAGE_SIZE) / 8;
        PHYS_MEM_BASE_ADR = base as u64;
        BITMAP = BitMap::new(bitmap_base as *mut u8, bitmap_len);
        info!("alloctable physical pool starting at address '0x{base:016X}' with a page count of '{}' and a page size of '0x{PAGE_SIZE:X}'", bitmap_len * 8);
        debug!(
            "original free memory section #{index} with base '0x{:016X}' and size '0x{:X}' is now reserved for the physical memory pool",
            entry.base, entry.len
        );
        found = true;
        break;
    }
    if !found {
        panic!("unable to find sufficient physical memory")
    }
}
