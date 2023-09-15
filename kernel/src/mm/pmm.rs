use crate::arch::{padr, vadr};
use crate::boot::BootInfo;
use crate::util::adr::{PhysAdr, VirtAdr};
use crate::util::locked::Locked;
use allocators::intrustive::free_list::IntrusiveFreeList;
use core::alloc::Layout;
use core::slice;

pub const PAGE_EXP: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_EXP;

static mut HHDM_BASE: VirtAdr = VirtAdr::null();
static mut HHDM_PG_CNT: usize = 0;

pub fn hhdm_base() -> VirtAdr {
    unsafe { HHDM_BASE }
}

pub fn hhdm_len() -> usize {
    unsafe { HHDM_PG_CNT * PAGE_SIZE }
}

pub unsafe fn hhdm_to_phys(adr: VirtAdr) -> PhysAdr {
    debug_assert!(adr.adr() >= hhdm_base().adr(), "invalid hhdm address");
    PhysAdr::new(adr.adr() - hhdm_base().adr())
}

pub unsafe fn phys_to_hhdm(adr: PhysAdr) -> VirtAdr {
    debug_assert!(adr.adr() < hhdm_base().adr(), "address is already hhdm");
    VirtAdr::new(adr.adr() + hhdm_base().adr())
}

static ALLOCATOR: Locked<IntrusiveFreeList<PAGE_SIZE>> = Locked::new(IntrusiveFreeList::new());

#[repr(C, align(4096))]
pub struct Page([u8; PAGE_SIZE]);

#[derive(Clone)]
pub struct PagePtr(*mut Page, usize);

impl PagePtr {
    #[inline]
    pub unsafe fn from_parts(ptr: *mut Page, count: usize) -> Self {
        Self(ptr, count)
    }

    #[inline]
    pub unsafe fn from_parts_hhdm(ptr: *mut Page, count: usize) -> Self {
        Self(ptr, count)
    }

    #[inline]
    fn ptr(&self) -> *const Page {
        self.0
    }

    #[inline]
    pub fn page_count(&self) -> usize {
        self.1
    }

    #[inline]
    pub fn byte_size(&self) -> usize {
        self.page_count() << PAGE_EXP
    }

    #[inline]
    pub fn virt(&self) -> VirtAdr {
        VirtAdr::new(self.0 as vadr)
    }

    #[inline]
    pub fn phys(&self) -> PhysAdr {
        let padr = self.ptr() as padr - unsafe { HHDM_BASE.adr() };
        PhysAdr::new(padr)
    }
}

#[must_use = "unused allocation causes memory leak"]
#[inline]
pub fn alloc_pages(count: usize) -> PagePtr {
    let mut lock = ALLOCATOR.lock();
    let alloc = match lock.alloc_layout(Layout::new::<Page>().repeat(count).unwrap().0) {
        Ok(ok) => ok,
        Err(e) => {
            panic!("failed to allocate {count} pages in the PMM with error '{e}'")
        }
    };
    PagePtr(alloc as *mut Page, count)
}

#[must_use = "unused allocation causes memory leak"]
#[inline]
pub fn alloc_pages_zeroed(count: usize) -> PagePtr {
    let ptr = alloc_pages(count);
    let virt_ptr = ptr.virt().ptr();
    unsafe { virt_ptr.write_bytes(0, count << PAGE_EXP) };
    PagePtr(virt_ptr as *mut Page, count)
}

#[inline]
pub fn free_pages(pages: PagePtr) {
    let mut lock = ALLOCATOR.lock();
    let ptr = pages.virt().ptr();
    let count = pages.1;
    let layout = Layout::new::<Page>().repeat(count).unwrap().0;
    lock.free_layout(ptr, layout);
}

pub unsafe fn init(boot_info: &mut BootInfo) {
    HHDM_BASE = VirtAdr::new(boot_info.hhdm.offset);
    info!("HHDM base at 0x{:016x}", HHDM_BASE.adr());
    let mmap = &mut boot_info.mmap;
    let count = mmap.entry_count;
    let entries = slice::from_raw_parts_mut(mmap.entries.as_ptr(), count as usize);
    let mut found = false;
    let mut highest_adr = 0;
    let mut lock = ALLOCATOR.lock();
    for (_index, entry) in entries.iter_mut().enumerate() {
        if entry.typ != limine::LimineMemoryMapEntryType::Usable {
            continue;
        }
        let base = entry.base;
        let len = entry.len;
        let end_adr = base + len;
        if end_adr > highest_adr {
            highest_adr = end_adr;
        }
        found = true;
        lock.push_region_unchecked(phys_to_hhdm(PhysAdr::new(base)).ptr(), len as usize);
        info!("PMM region: ");
        info!("    base: 0x{base:016x}");
        info!("    len:  0x{len:x}");
        entry.typ = limine::LimineMemoryMapEntryType::Reserved;
    }
    if !found {
        panic!("unable to reserve any memory for pmm")
    }
    HHDM_PG_CNT = highest_adr as usize / PAGE_SIZE;
}
