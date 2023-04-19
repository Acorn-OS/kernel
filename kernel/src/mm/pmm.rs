use crate::boot::BootInfo;
use core::ptr::null_mut;
use core::slice;

pub const PAGE_EXP: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_EXP;

type AllocatorTy = bitmap::BitMapPtrAllocator<PAGE_EXP>;

static mut ALLOCATED_PAGES: usize = 0;
static ALLOCATOR: AllocatorTy = unsafe { AllocatorTy::new(null_mut(), 0, null_mut()) };

#[repr(C, align(4096))]
pub struct Page([u8; PAGE_SIZE]);

pub fn alloc_pages(count: usize) -> *const Page {
    ALLOCATOR
        .alloc_pages(count)
        .expect("failed to allocate physical memory") as *const Page
}

pub fn alloc_pages_zeroed(count: usize) -> *const Page {
    let ptr = ALLOCATOR
        .alloc_pages(count)
        .expect("failed to allocate physical memory");
    unsafe { ptr.write_bytes(0, count << PAGE_EXP) };
    ptr as *mut _
}

pub fn free_pages(ptr: *const Page, count: usize) {
    ALLOCATOR.free_pages(ptr as *mut _, count)
}

pub unsafe fn init(boot_info: &mut BootInfo) {
    fn align_floor(val: usize, align: usize) -> usize {
        val.div_floor(align) * align
    }

    fn align_ceil(val: usize, align: usize) -> usize {
        val.div_ceil(align) * align
    }
    let mmap = &mut boot_info.mmap;
    let count = mmap.entry_count;
    let entries = slice::from_raw_parts_mut(mmap.entries.as_ptr(), count as usize);
    let mut found = false;
    for (_index, entry) in entries.iter_mut().enumerate() {
        if entry.typ != limine::LimineMemoryMapEntryType::Usable {
            continue;
        }
        let base = entry.base;
        let len = entry.len;
        let entry_pages = len >> PAGE_EXP;
        let resv_bitmap_pages = entry_pages.div_ceil(PAGE_SIZE as u64 * 8);
        let alloc_pages = entry_pages - resv_bitmap_pages;
        let bitmap_pages = resv_bitmap_pages;
        if alloc_pages > 256 {
            found = true;
            ALLOCATOR.init(
                base as *mut u8,
                len as usize,
                (base + bitmap_pages * PAGE_SIZE as u64) as *mut u8,
            );
            ALLOCATED_PAGES = alloc_pages as usize;
            entry.typ = limine::LimineMemoryMapEntryType::Reserved;
            break;
        }
    }
    if !found {
        panic!("unable to reserve memory for pmm")
    }
}
