use crate::mm::pmm;
use crate::util::adr::{PhysAdr, VirtAdr};
use core::arch::asm;
use core::mem::size_of;
use core::ptr::NonNull;

use super::vadr;

assert_eq_size!(usize, u64);

pub const SMALL_PAGE_SIZE: usize = 1 << 12;
pub const MEDIUM_PAGE_SIZE: usize = 1 << 21;
pub const LARGE_PAGE_SIZE: usize = 1 << 30;

pub const PAGE_SIZE: usize = SMALL_PAGE_SIZE as usize;

#[inline]
fn divide_virt_adr(virt: VirtAdr) -> (usize, usize, usize, usize, u64) {
    let virt = virt.adr();
    (
        (virt as usize >> 39) & 0x1FF,
        (virt as usize >> 30) & 0x1FF,
        (virt as usize >> 21) & 0x1FF,
        (virt as usize >> 12) & 0x1FF,
        virt & 0xFFF,
    )
}

bitfield! {
    #[derive(Clone, Copy)]
    #[repr(C, align(8))]
    pub struct PageMapEntry(u64): Debug {
        pub(super) p :bool @ 0,
        rw: bool @ 1,
        us: bool @ 2,
        pwt: bool @ 3,
        pcd: bool @ 4,
        a: bool @ 5,
        ps: bool @ 7,
        pub(super) resv: bool @ 9,
        inner_adr: u64 @ 12..=51,
        xd: bool @ 63,
    }
}
const_assert!(size_of::<PageMapEntry>() == 8);

impl PageMapEntry {
    pub fn new(phys_adr: PhysAdr) -> Self {
        let mut entry = Self(0);
        entry.set_adr(phys_adr);
        entry
    }

    pub fn adr(&self) -> PhysAdr {
        PhysAdr::new(((((self.inner_adr() << 12) as i64) << 16) >> 16) as u64)
    }

    pub fn set_adr(&mut self, adr: PhysAdr) {
        self.set_inner_adr(adr.adr() >> 12)
    }

    pub fn present(&self) -> bool {
        self.p()
    }

    fn modify_with_flags(&mut self, flags: Flags) -> Self {
        self.0 |= flags.0 as u64 & 0xfff;
        self.set_xd(flags.has(Flags::XD));
        *self
    }
}

const PAGE_MAP_ENTRIES: usize = 512;

#[repr(C, align(4096))]
struct PageMap {
    entries: [PageMapEntry; PAGE_MAP_ENTRIES],
}
const_assert!(size_of::<PageMap>() == PAGE_SIZE);
const_assert!(PAGE_SIZE == pmm::PAGE_SIZE);

#[derive(Clone, Copy)]
pub struct PageMapPtr(VirtAdr);

impl core::fmt::Debug for PageMapPtr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("page map ptr({})", self.adr(),))
    }
}

impl PageMapPtr {
    /// # Safety
    /// gives back an invalid page map ptr.
    pub const unsafe fn nullptr() -> Self {
        Self::from_vadr(VirtAdr::new(0))
    }

    fn new_alloc() -> Self {
        Self::new(pmm::alloc_pages(1).virt().ptr() as *mut _)
    }

    fn new(ptr: *mut PageMap) -> Self {
        Self(VirtAdr::new(ptr as vadr))
    }

    const fn from_vadr(adr: VirtAdr) -> Self {
        Self(adr)
    }

    unsafe fn ptr(&self) -> *mut PageMap {
        self.0.ptr() as *mut _
    }

    unsafe fn get(&self) -> &'static mut PageMap {
        &mut *self.ptr()
    }

    pub fn adr(self) -> VirtAdr {
        self.0
    }

    pub unsafe fn map(self, virt: VirtAdr, pages: usize, phys: PhysAdr, flags: Flags) {
        self.get().map(virt, pages, phys, flags);
    }

    pub unsafe fn unmap(self, _virt: VirtAdr, _pages: usize) {}

    pub fn to_phys_adr(self) -> PhysAdr {
        unsafe { pmm::hhdm_to_phys(self.adr()) }
    }

    pub fn entry(self, index: usize) -> &'static mut PageMapEntry {
        debug_assert!(index < 512);
        let index = index & 0x1FF;
        unsafe { &mut self.get().entries[index] }
    }
}

mod get {
    use super::*;

    pub unsafe fn allocate(page_map: PageMapPtr, index: usize, flags: Flags) -> PageMapPtr {
        let entry = page_map.entry(index);
        if entry.p() && entry.ps() {
            panic!("reallocating page")
        } else if !entry.p() && !entry.resv() {
            let new = PageMapPtr::new_alloc();
            *entry = PageMapEntry::new(new.to_phys_adr()).modify_with_flags(flags);
            new
        } else {
            PageMapPtr::from_vadr(pmm::phys_to_hhdm(entry.adr()))
        }
    }

    pub unsafe fn get(page_map: PageMapPtr, index: usize) -> Option<PageMapPtr> {
        let entry = page_map.entry(index);
        if entry.p() && entry.ps() {
            panic!("reallocating page")
        } else if !entry.p() {
            None
        } else {
            Some(PageMapPtr::from_vadr(pmm::phys_to_hhdm(entry.adr())))
        }
    }
}

unsafe fn set(page_map: PageMapPtr, index: usize, phys: PhysAdr, flags: Flags) {
    let entry = page_map.entry(index);
    entry.set_adr(phys);
    entry.modify_with_flags(flags);
}

unsafe fn get(page_map: PageMapPtr, index: usize) -> NonNull<PageMapEntry> {
    NonNull::new_unchecked(page_map.entry(index) as *mut _)
}

impl PageMap {
    pub const fn new() -> Self {
        Self {
            entries: [PageMapEntry(0); 512],
        }
    }

    pub unsafe fn install(&mut self) {
        PageMap::install_ptr(self as *const _);
    }

    unsafe fn install_ptr(ptr: *const Self) {
        unsafe {
            asm!(
                "mov cr3, rax",
                in("rax") ptr as u64,
                options(nostack)
            );
        }
    }

    pub unsafe fn get(&mut self, virt: VirtAdr) -> Option<NonNull<PageMapEntry>> {
        let ptr = self.as_ptr();
        let (d0, d1, d2, d3, _) = divide_virt_adr(virt);
        Some(get(get::get(get::get(get::get(ptr, d0)?, d1)?, d2)?, d3))
    }

    unsafe fn map(&mut self, mut virt: VirtAdr, pages: usize, mut phys: PhysAdr, flags: Flags) {
        let ptr = self.as_ptr();
        fn mask(size: usize) -> usize {
            !(size - 1)
        }
        let get_flags = Flags::PRESENT | Flags::RW | Flags::USER;
        let set_flags = flags;
        if flags.has(Flags::SIZE_LARGE) {
            for _ in 0..pages {
                let (d0, d1, _, _, _) = divide_virt_adr(virt);
                set(
                    get::allocate(ptr, d0, get_flags),
                    d1,
                    phys.mask(mask(LARGE_PAGE_SIZE)),
                    set_flags | Flags::PS,
                );
                virt = virt.add(LARGE_PAGE_SIZE);
                phys = phys.add(LARGE_PAGE_SIZE);
            }
        } else if flags.has(Flags::SIZE_MEDIUM) {
            for _ in 0..pages {
                let (d0, d1, d2, _, _) = divide_virt_adr(virt);
                set(
                    get::allocate(get::allocate(ptr, d0, get_flags), d1, get_flags),
                    d2,
                    phys.mask(mask(MEDIUM_PAGE_SIZE)),
                    set_flags | Flags::PS,
                );
                virt = virt.add(MEDIUM_PAGE_SIZE);
                phys = phys.add(MEDIUM_PAGE_SIZE);
            }
        } else {
            for _ in 0..pages {
                let (d0, d1, d2, d3, _) = divide_virt_adr(virt);
                set(
                    get::allocate(
                        get::allocate(get::allocate(ptr, d0, get_flags), d1, get_flags),
                        d2,
                        get_flags,
                    ),
                    d3,
                    phys.mask(mask(SMALL_PAGE_SIZE)),
                    set_flags,
                );
                virt = virt.add(SMALL_PAGE_SIZE);
                phys = phys.add(SMALL_PAGE_SIZE);
            }
        }
    }

    fn as_ptr(&mut self) -> PageMapPtr {
        PageMapPtr::new(self as *mut PageMap)
    }
}

static mut KERNEL_PAGE_MAP_PTR: PageMapPtr = unsafe { PageMapPtr::nullptr() };

pub unsafe fn init() {
    // Map the kernel map.
    KERNEL_PAGE_MAP_PTR = PageMapPtr::new_alloc();
    for i in 256..PAGE_MAP_ENTRIES {
        set(
            KERNEL_PAGE_MAP_PTR,
            i,
            pmm::alloc_pages_zeroed(1).phys(),
            Flags::PRESENT | Flags::RW,
        );
    }
}

#[derive(Clone, Copy)]
#[must_use]
#[repr(transparent)]
pub struct Flags(u32);

impl Flags {
    pub const NONE: Flags = Flags(0);
    pub const PRESENT: Flags = Flags(1 << 0);
    pub const RW: Flags = Flags(1 << 1);
    pub const USER: Flags = Flags(1 << 2);
    const PS: Flags = Flags(1 << 7);
    const RESV: Flags = Flags(1 << 9);

    pub const SIZE_LARGE: Flags = Flags(1 << 16);
    pub const SIZE_MEDIUM: Flags = Flags(1 << 17);
    pub const XD: Flags = Flags(1 << 18);

    pub const fn merge(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn has(self, flags: Flags) -> bool {
        self.0 & flags.0 == flags.0
    }
}

impl core::ops::BitOr for Flags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.merge(rhs)
    }
}

pub fn kernel_page_map() -> PageMapPtr {
    unsafe { KERNEL_PAGE_MAP_PTR }
}

pub unsafe fn new_userland_page_map() -> PageMapPtr {
    let ptr = PageMapPtr::from_vadr(pmm::alloc_pages_zeroed(1).virt());
    for i in 256..PAGE_MAP_ENTRIES {
        *ptr.entry(i) = KERNEL_PAGE_MAP_PTR.entry(i).clone();
    }
    ptr
}

pub unsafe fn install(map: PageMapPtr) {
    PageMap::install_ptr(map.to_phys_adr().adr() as *mut PageMap)
}

pub unsafe fn get_page_entry(map: PageMapPtr, virt: VirtAdr) -> Option<NonNull<PageMapEntry>> {
    map.get().get(virt)
}

pub unsafe fn map(map: PageMapPtr, virt: VirtAdr, pages: usize, phys: PhysAdr, flags: Flags) {
    map.map(virt, pages, phys, flags)
}

pub unsafe fn unmap(map: PageMapPtr, _virt: VirtAdr, _pages: usize) {
    map.unmap(_virt, _pages)
}
