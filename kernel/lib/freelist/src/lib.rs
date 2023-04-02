#![no_std]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(int_roundings)]
#![feature(const_trait_impl)]
#![feature(const_default_impls)]

#[macro_use]
extern crate alloc;

use alloc::alloc::{AllocError, Allocator, Global};
use alloc::slice;
use core::alloc::{GlobalAlloc, Layout};
use core::fmt::Debug;
use core::ptr::{null_mut, NonNull};
use spin::Mutex;

#[derive(Debug)]
pub enum Error {
    InsufficientSpace,
    AllocError(AllocError),
}

pub type Result<T> = core::result::Result<T, Error>;

impl From<AllocError> for Error {
    fn from(value: AllocError) -> Self {
        Self::AllocError(value)
    }
}

pub struct Node {
    start: u64,
    len: usize,
    next: *mut Self,
}

impl Node {
    pub fn new(start: u64, len: usize) -> Self {
        Self {
            start,
            len,
            next: null_mut(),
        }
    }
}

pub struct FreeList<A: Allocator = Global> {
    head: *mut Node,
    tail: *mut Node,
    allocator: A,
}

unsafe impl<A: Allocator> Send for FreeList<A> {}

impl Debug for FreeList<Global> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use alloc::string::String;
        f.debug_struct("FreeLists")
            .field("regions", &{
                let mut vec = vec![];
                let mut head = self.head;
                while !head.is_null() {
                    unsafe {
                        vec.push(format!(
                            "{:016X}:{:016X}",
                            (*head).start,
                            (*head).start + (*head).len as u64,
                        ));
                        head = (*head).next;
                    }
                }
                format!(
                    "[{}]",
                    vec.iter()
                        .fold(String::from(" "), |a, v| format!("{a}{v}, "))
                )
            })
            .finish()
    }
}

impl FreeList<Global> {
    pub const fn new() -> Self {
        unsafe { Self::with_allocator(Global) }
    }
}

impl<A: Allocator> FreeList<A> {
    pub const unsafe fn with_allocator(a: A) -> Self {
        Self {
            head: null_mut(),
            tail: null_mut(),
            allocator: a,
        }
    }

    pub fn push_region(&mut self, adr: u64, len: usize) -> Result<()> {
        let layout = Layout::new::<Node>();
        let alloc = self.allocator.allocate(layout)?.as_ptr().as_mut_ptr() as *mut Node;
        unsafe {
            alloc.write(Node {
                start: adr,
                len,
                next: null_mut(),
            })
        };
        unsafe { self.push_node_raw(alloc) };
        Ok(())
    }

    pub unsafe fn push_node_raw(&mut self, node: *mut Node) {
        if self.tail.is_null() {
            self.tail = node;
        } else {
            unsafe { (*self.tail).next = node };
            self.tail = node;
        }
        if self.head.is_null() {
            self.head = self.tail;
        }
    }

    pub fn alloc_aligned_bytes(&mut self, align: usize, len: usize) -> Result<*mut u8> {
        let mut previous = null_mut() as *mut Node;
        let mut head = self.head;
        unsafe {
            while !head.is_null() {
                let mut cur_node = head.read();
                let aligned_start = (cur_node.start + align as u64 - 1) & !(align - 1) as u64;
                let removed_len = len + (aligned_start - cur_node.start) as usize;
                if cur_node.len >= removed_len {
                    cur_node.start += removed_len as u64;
                    cur_node.len -= removed_len;
                    if cur_node.len > 0 {
                        head.write(cur_node);
                    } else {
                        if !previous.is_null() {
                            (*previous).next = (*head).next;
                        } else {
                            self.head = (*head).next;
                        }
                        self.allocator.deallocate(
                            NonNull::new_unchecked(head as *mut u8),
                            Layout::new::<Self>(),
                        );
                    }
                    return Ok(aligned_start as *mut u8);
                } else {
                    previous = head;
                    head = cur_node.next;
                }
            }
        }
        Err(Error::InsufficientSpace)
    }

    pub fn alloc<T>(&mut self) -> Result<*mut T> {
        let layout = Layout::new::<T>();
        self.alloc_aligned_bytes(layout.align(), layout.size())
            .map(|v| v as *mut T)
    }

    pub fn free_bytes(&mut self, ptr: *mut u8, len: usize) -> Result<()> {
        let begin = ptr as u64;
        let end = begin + len as u64;
        let mut head = self.head;
        unsafe {
            while !head.is_null() {
                let mut val = head.read();
                let cur_begin = val.start;
                let cur_end = val.start + val.len as u64;
                if cur_begin == end {
                    val.start = begin;
                    val.len += len;
                    head.write(val);
                    return Ok(());
                } else if cur_end == begin {
                    val.len += len;
                    head.write(val);
                    return Ok(());
                }
                head = val.next;
            }
        }
        self.push_region(begin, len)
    }

    pub fn free<T>(&mut self, ptr: *mut T) -> Result<()> {
        let layout = Layout::new::<T>();
        self.free_bytes(ptr as *mut u8, layout.size())
    }
}

pub struct FreeListAllocator<A: Allocator = Global>(Mutex<FreeList<A>>);

impl Debug for FreeListAllocator<Global> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0.lock(), f)
    }
}

impl<A: Allocator> FreeListAllocator<A> {
    pub const unsafe fn new(a: A) -> Self {
        Self(Mutex::new(FreeList::with_allocator(a)))
    }

    pub unsafe fn push_node(&self, node: *mut Node) {
        self.0.lock().push_node_raw(node);
    }

    pub fn push_region(&self, adr: u64, len: usize) -> Result<()> {
        self.0.lock().push_region(adr, len)
    }
}

unsafe impl<A: Allocator + Default> Allocator for FreeListAllocator<A> {
    fn allocate(&self, layout: Layout) -> core::result::Result<NonNull<[u8]>, AllocError> {
        match self
            .0
            .lock()
            .alloc_aligned_bytes(layout.align(), layout.size())
        {
            Ok(ptr) if ptr.is_null() => Err(AllocError),
            Ok(ptr) => Ok(NonNull::new(unsafe {
                slice::from_raw_parts_mut::<u8>(ptr, layout.size()) as *mut [u8]
            })
            .expect("expected a non-null ptr")),
            Err(err) => Err(match err {
                Error::InsufficientSpace => AllocError,
                Error::AllocError(err) => err,
            }),
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        match self.0.lock().free_bytes(ptr.as_ptr(), layout.size()) {
            Ok(_) => {}
            Err(err) => panic!("failed to deallocate in FreeList with error '{err:?}'"),
        }
    }
}

unsafe impl<A: Allocator + Default> GlobalAlloc for FreeListAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Ok(res) = self.allocate(layout) {
            res.as_ptr().as_mut_ptr()
        } else {
            null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() {
            panic!("attempting to deallocate a nullptr");
        }
        self.deallocate(NonNull::new(ptr).unwrap_unchecked(), layout);
    }
}
