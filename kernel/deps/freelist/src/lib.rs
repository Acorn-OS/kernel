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

fn node_alloc<A: Allocator>(node: Node, a: &A) -> Result<*mut Node> {
    let layout = Layout::new::<Node>();
    let alloc = a.allocate(layout)?.as_ptr().as_mut_ptr() as *mut Node;
    unsafe { alloc.write(node) };
    Ok(alloc)
}

fn node_free<A: Allocator>(node: *mut Node, a: &A) {
    let layout = Layout::new::<Node>();
    unsafe { a.deallocate(NonNull::new_unchecked(node as *mut u8), layout) }
}

pub struct FreeList<A: Allocator = Global> {
    head: *mut Node,
    allocator: A,
}

unsafe impl<A: Allocator> Send for FreeList<A> {}

impl<A: Allocator> Debug for FreeList<A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use alloc::string::String;
        f.debug_struct("FreeLists")
            .field("regions", &{
                let mut vec = vec![];
                let mut head = self.head;
                while !head.is_null() {
                    unsafe {
                        vec.push(format!(
                            "{:016X}:{:016X} ({:X})",
                            (*head).start,
                            (*head).start + (*head).len as u64,
                            (*head).len,
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
            allocator: a,
        }
    }

    pub fn push_region(&mut self, adr: u64, len: usize) -> Result<()> {
        let node = node_alloc(Node::new(adr, len), &self.allocator)?;
        unsafe { self.push_node_raw(node) };
        Ok(())
    }

    pub unsafe fn push_node_raw(&mut self, node: *mut Node) {
        if self.head.is_null() {
            self.head = node;
        } else {
            let prev_head = self.head;
            self.head = node;
            (*self.head).next = prev_head;
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

    unsafe fn insert_node(&mut self, node: *mut Node) {
        let mut cur = self.head;
        let mut previous = null_mut() as *mut Node;
        let begin = (*node).start;
        let len = (*node).len;
        let end = begin + len as u64;
        while !cur.is_null() {
            let cur_begin = (*cur).start;
            let cur_end = (*cur).start + (*cur).len as u64;
            if cur_begin == end {
                if previous.is_null() {
                    self.head = (*cur).next;
                } else {
                    (*previous).next = (*cur).next;
                };
                (*cur).start = begin;
                (*cur).len += len;
                (*cur).next = null_mut();
                self.insert_node(cur);
                node_free(node, &self.allocator);
                return;
            } else if cur_end == begin {
                if previous.is_null() {
                    self.head = (*cur).next;
                } else {
                    (*previous).next = (*cur).next;
                };
                (*cur).len += len;
                (*cur).next = null_mut();
                self.insert_node(cur);
                node_free(node, &self.allocator);
                return;
            }
            previous = cur;
            cur = (*cur).next;
        }
        self.push_node_raw(node);
    }

    pub fn free_bytes(&mut self, ptr: *mut u8, len: usize) -> Result<()> {
        let node = node_alloc(Node::new(ptr as u64, len), &self.allocator)?;
        unsafe {
            self.insert_node(node);
        }
        Ok(())
    }

    pub fn free<T>(&mut self, ptr: *mut T) -> Result<()> {
        let layout = Layout::new::<T>();
        self.free_bytes(ptr as *mut u8, layout.size())
    }

    pub fn reserve_bytes(&mut self, adr: u64, count: usize) -> Result<()> {
        let start = adr;
        let end = start + count as u64;
        let mut cur = self.head;
        let mut prev = null_mut() as *mut Node;
        unsafe {
            while !cur.is_null() {
                let cur_start = (*cur).start;
                let cur_len = (*cur).len;
                let cur_end = cur_start + (cur_len.saturating_sub(1)) as u64;
                if start >= cur_start && end <= cur_end {
                    let left_beg = cur_start;
                    let left_len = start - left_beg;
                    let right_beg = end;
                    let right_len = cur_end - right_beg;
                    if prev.is_null() {
                        self.head = (*cur).next;
                    } else {
                        (*prev).next = (*cur).next;
                    }
                    node_free(cur, &self.allocator);
                    if left_len > 0 {
                        self.push_region(left_beg, left_len as usize)?;
                    }
                    if right_len > 0 {
                        self.push_region(right_beg, right_len as usize)?;
                    }
                    break;
                } else if start >= cur_start && start < cur_end && end > cur_end {
                    let len = cur_len - (start - cur_start) as usize;
                    (*cur).len = len;
                    if len <= 0 {
                        if prev.is_null() {
                            (*prev).next = (*cur).next;
                        } else {
                            self.head = (*cur).next;
                        }
                        node_free(cur, &self.allocator);
                    }
                    self.reserve_bytes(cur_end, (end - cur_end) as usize)?;
                    break;
                } else if start < cur_start && end >= cur_start && end < cur_end {
                    let len = cur_len - (end - cur_start) as usize;
                    (*cur).start = end;
                    (*cur).len = len;
                    if len <= 0 {
                        if prev.is_null() {
                            (*prev).next = (*cur).next;
                        } else {
                            self.head = (*cur).next;
                        }
                        node_free(cur, &self.allocator);
                    }
                    self.reserve_bytes(start, (cur_start - start) as usize)?;
                    break;
                }
                prev = cur;
                cur = (*cur).next;
            }
        }
        Ok(())
    }
}

pub struct FreeListAllocator<A: Allocator = Global>(Mutex<FreeList<A>>);

unsafe impl<A: Allocator + Sync> Sync for FreeListAllocator<A> {}

impl<A: Allocator> Debug for FreeListAllocator<A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0.lock(), f)
    }
}

impl FreeListAllocator<Global> {
    pub const fn new() -> Self {
        Self(Mutex::new(FreeList::new()))
    }
}

impl<A: Allocator> FreeListAllocator<A> {
    pub const unsafe fn with_allocator(a: A) -> Self {
        Self(Mutex::new(FreeList::with_allocator(a)))
    }

    pub unsafe fn push_node(&self, node: *mut Node) {
        self.0.lock().push_node_raw(node);
    }

    pub fn push_region(&self, adr: u64, len: usize) -> Result<()> {
        self.0.lock().push_region(adr, len)
    }

    pub fn reserve_bytes(&self, adr: u64, count: usize) -> Result<()> {
        self.0.lock().reserve_bytes(adr, count)
    }
}

unsafe impl<A: Allocator> Allocator for FreeListAllocator<A> {
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
            Err(e) => panic!("deallocation in freelist with error '{e:?}'"),
        }
    }
}

unsafe impl<A: Allocator> GlobalAlloc for FreeListAllocator<A> {
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
