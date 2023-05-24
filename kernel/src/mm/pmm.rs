use crate::arch::{padr, vadr};
use crate::boot::BootInfo;
use crate::util::adr::{PhysAdr, VirtAdr};
use core::ptr::null_mut;
use core::slice;

pub const PAGE_EXP: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_EXP;

static mut HHDM_BASE: u64 = 0;
static mut SIZE: usize = 0;

pub fn hhdm_base() -> u64 {
    unsafe { HHDM_BASE }
}

pub fn page_cnt() -> usize {
    unsafe { SIZE }
}

pub unsafe fn hhdm_to_phys(adr: VirtAdr) -> PhysAdr {
    debug_assert!(adr.adr() >= hhdm_base(), "invalid hhdm address");
    PhysAdr::new(adr.adr() - hhdm_base())
}

pub unsafe fn phys_to_hhdm(adr: PhysAdr) -> VirtAdr {
    debug_assert!(adr.adr() < hhdm_base(), "address is already hhdm");
    VirtAdr::new(adr.adr() + hhdm_base())
}

type AllocatorTy = allocators::bitmap::BitMapPtrAllocator<PAGE_EXP>;

static ALLOCATOR: AllocatorTy = unsafe { AllocatorTy::new(null_mut(), 0, null_mut()) };

#[repr(C, align(4096))]
pub struct Page([u8; PAGE_SIZE]);

#[derive(Clone)]
pub struct PagePtr(*const Page, usize);

impl PagePtr {
    pub unsafe fn from_parts(ptr: *const Page, count: usize) -> Self {
        Self(ptr, count)
    }

    pub unsafe fn from_parts_hhdm(ptr: *mut Page, count: usize) -> Self {
        Self(
            hhdm_to_phys(VirtAdr::new(ptr as u64)).adr() as *mut _,
            count,
        )
    }

    fn ptr(&self) -> *const Page {
        self.0
    }

    pub fn page_count(&self) -> usize {
        self.1
    }

    pub fn byte_size(&self) -> usize {
        self.page_count() << PAGE_EXP
    }

    pub fn virt(&self) -> VirtAdr {
        let vadr = self.ptr() as vadr + unsafe { HHDM_BASE as vadr };
        VirtAdr::new(vadr)
    }

    pub fn phys(&self) -> PhysAdr {
        PhysAdr::new(self.ptr() as padr)
    }
}

#[must_use = "unused allocation causes memory leak"]
pub fn alloc_pages(count: usize) -> PagePtr {
    PagePtr(
        ALLOCATOR
            .alloc_pages(count)
            .expect("failed to allocate physical memory") as *const Page,
        count,
    )
}

#[must_use = "unused allocation causes memory leak"]
pub fn alloc_pages_zeroed(count: usize) -> PagePtr {
    let ptr = ALLOCATOR
        .alloc_pages(count)
        .expect("failed to allocate physical memory");
    unsafe { (ptr.add(HHDM_BASE as usize)).write_bytes(0, count << PAGE_EXP) };
    PagePtr(ptr as *const _, count)
}

pub fn free_pages(pages: PagePtr) {
    ALLOCATOR.free_pages(pages.ptr() as *mut _, pages.page_count())
}

pub unsafe fn init(boot_info: &mut BootInfo) {
    HHDM_BASE = boot_info.hhdm.offset;
    info!("HHDM base at 0x{HHDM_BASE:016x}");
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
            SIZE = alloc_pages as usize;
            found = true;
            ALLOCATOR.init(
                phys_to_hhdm(PhysAdr::new(base)).ptr(),
                len as usize,
                (base + bitmap_pages * PAGE_SIZE as u64) as *mut u8,
            );
            info!("PMM init: ");
            info!("    base: 0x{base:016x}");
            info!("    len:  0x{len:x}");
            entry.typ = limine::LimineMemoryMapEntryType::Reserved;
            break;
        }
    }
    if !found {
        panic!("unable to reserve memory for pmm")
    }
}
