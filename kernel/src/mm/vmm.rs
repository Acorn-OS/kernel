use crate::arch::cpuc;
use crate::arch::vm::{self, PageMap};
use crate::boot;
use crate::mm::pmm;
use alloc::slice;
use bitmap::BitMap;
use core::alloc::{AllocError, Allocator, Layout};
use core::cell::UnsafeCell;
use core::fmt::Debug;
use core::ptr::NonNull;

use super::hhdm;
use super::pmm::Page;

pub const PAGE_SIZE: usize = pmm::PAGE_SIZE;

struct NodeBitMapAllocator {
    bitmap: UnsafeCell<BitMap<3, { PAGE_SIZE >> 3 }>>,
    base: *mut u8,
}

unsafe impl Allocator for NodeBitMapAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        assert_eq_align!(freelist::Node, u64);
        let bitmap = unsafe { &mut *self.bitmap.get() };
        let pages = layout.size().div_ceil(bitmap.page_size());
        let index = bitmap.get_first_empty(pages).ok_or(AllocError)?;
        for i in 0..pages {
            bitmap.alloc(index + i)
        }
        let ptr = unsafe { self.base.add(index * (*self.bitmap.get()).page_size()) };
        Ok(unsafe {
            NonNull::new_unchecked(slice::from_raw_parts_mut(ptr, pages * bitmap.page_size()))
        })
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let bitmap = &mut *self.bitmap.get();
        let index = ((ptr.as_ptr() as usize) - self.base as usize) / bitmap.page_size();
        let count = layout.size() / bitmap.page_size();
        for i in 0..count {
            bitmap.free(index + i)
        }
    }
}

type AllocatorTy = freelist::FreeList<NodeBitMapAllocator>;

pub struct VirtualMemory {
    root_map: *mut PageMap,
    allocator: AllocatorTy,
}

impl Debug for VirtualMemory {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VirtualMemory")
            .field("root_map", &self.root_map)
            .field("allocator", &self.allocator)
            .finish()
    }
}

impl VirtualMemory {
    pub const PAGE_SIZE: usize = PAGE_SIZE;

    pub fn alloc_pages(&mut self, pages: usize) -> *mut Page {
        let virt = self
            .allocator
            .alloc_aligned_bytes(Self::PAGE_SIZE, pages * Self::PAGE_SIZE)
            .expect("failed to allocate virtual memory");
        unsafe { vm::resv_pages(self.root_map, virt as u64, pages) };
        virt as *mut _
    }

    pub fn alloc_pages_with_base_virt(&mut self, pages: usize, virt: u64) {
        self.allocator
            .reserve_bytes(virt, pages * PAGE_SIZE)
            .expect("failed to reserve virtual memory");
        unsafe { vm::resv_pages(self.root_map, virt, pages) };
    }

    pub fn map_pages(&mut self, pages: usize, phys: u64) -> *mut Page {
        let virt = self
            .allocator
            .alloc_aligned_bytes(Self::PAGE_SIZE, pages * Self::PAGE_SIZE)
            .expect("failed to allocate virtual memory");
        unsafe { vm::alloc_pages(self.root_map, virt as u64, pages, phys) };
        virt as *mut _
    }

    pub unsafe fn map_pages_raw(&mut self, pages: usize, virt: u64, phys: u64) {
        self.allocator
            .reserve_bytes(virt, pages * Self::PAGE_SIZE)
            .expect("failed to reserve virtual memory");
        vm::alloc_pages(self.root_map, virt, pages, phys);
    }

    pub fn free_pages(_ptr: *mut u8, _pages: usize) {
        unimplemented!()
    }

    pub unsafe fn install(&self) {
        vm::install(self.root_map)
    }
}

pub fn new_kernel() -> VirtualMemory {
    let mut map = VirtualMemory {
        allocator: unsafe {
            AllocatorTy::with_allocator(NodeBitMapAllocator {
                bitmap: UnsafeCell::new(BitMap::new()),
                base: hhdm::to_virt(pmm::alloc_pages(1)) as *mut _,
            })
        },
        root_map: vm::new_page_map(),
    };
    // Identity map the initial 4GiB of physical memory.
    unsafe { vm::alloc_large_pages(map.root_map, 0, 4, 0) };
    // Memory map the initial 4GiB of HHDM.
    unsafe { vm::alloc_large_pages(map.root_map, hhdm::base(), 4, 0) };
    // Push allocatable region.
    map.allocator
        .push_region(0xffff800000000000, 1 << 47)
        .expect("failed to push map allocator region");
    // Reserve HHDM direct map.
    map.allocator
        .reserve_bytes(hhdm::base(), hhdm::size())
        .expect("failed to reserve bytes for kernel vmm");
    // Map kernel to high address.
    let s4kib = 4 << 10;
    let page_count = (30 << 20) / s4kib;
    let phys_adr = unsafe { boot::info().kernel_address.physical_base };
    let virt_adr = 0xffffffff80000000;
    unsafe { map.map_pages_raw(page_count, virt_adr, phys_adr) };
    map
}

pub fn new_userland() -> VirtualMemory {
    todo!()
}

fn get() -> *mut VirtualMemory {
    let ptr = cpuc::get();
    debug_assert!(!ptr.is_null());
    let ptr = unsafe { (*ptr).vmm() };
    debug_assert!(!ptr.is_null());
    ptr
}

pub fn alloc(count: usize) -> *mut Page {
    let ptr = get();
    unsafe { (*ptr).alloc_pages(count) }
}

pub fn free(_ptr: *mut Page, _count: usize) {
    unimplemented!()
}
