#![no_std]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(int_roundings)]

#[cfg(test)]
mod test;

#[cfg(test)]
extern crate std;

#[macro_use]
extern crate alloc;

use alloc::alloc::{AllocError, Allocator, Global};
use alloc::slice;
use core::alloc::Layout;
use core::fmt::Debug;
use core::ptr::{null_mut, NonNull};

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

struct Node {
    start: u64,
    len: usize,
    next: *mut Self,
}

impl Node {
    fn new(start: u64, len: usize) -> Self {
        Self {
            start,
            len,
            next: null_mut(),
        }
    }

    fn alloc(node: Self) -> Result<*mut Self> {
        let layout = Layout::new::<Self>();
        let ptr = Global::default().allocate(layout)?.as_ptr().as_mut_ptr() as *mut Self;
        unsafe { ptr.write(node) };
        Ok(ptr)
    }

    fn pretty_print(&self) -> alloc::string::String {
        format!("{:016X}:{:016X}", self.start, self.start + self.len as u64)
    }
}

pub struct FreeList {
    head: *mut Node,
    tail: *mut Node,
}

unsafe impl Send for FreeList {}

impl Debug for FreeList {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use alloc::string::String;
        f.debug_struct("FreeLists")
            .field("regions", &{
                let mut vec = vec![];
                let mut head = self.head;
                while !head.is_null() {
                    let val = unsafe { head.read() };
                    vec.push(val.pretty_print());
                    head = val.next;
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

impl FreeList {
    pub fn new() -> Self {
        Self {
            head: null_mut(),
            tail: null_mut(),
        }
    }

    pub fn push_region(&mut self, adr: u64, len: usize) -> Result<()> {
        let alloc = Node::alloc(Node::new(adr, len))?;
        if self.tail.is_null() {
            self.tail = alloc;
        } else {
            unsafe { (*self.tail).next = alloc };
            self.tail = alloc;
        }
        if self.head.is_null() {
            self.head = self.tail;
        }
        Ok(())
    }

    pub fn alloc_aligned_bytes(&mut self, align: usize, len: usize) -> Result<*mut u8> {
        let mut previous = null_mut() as *mut Node;
        let mut head = self.head;
        unsafe {
            while !head.is_null() {
                let mut cur_node = head.read();
                let aligned_start = (cur_node.start + align as u64 - 1) / align as u64;
                let removed_len = len + (aligned_start - cur_node.start) as usize;
                if cur_node.len >= removed_len {
                    let old_start = cur_node.start;
                    cur_node.start = aligned_start + len as u64;
                    cur_node.len -= removed_len;
                    if cur_node.len > 0 {
                        head.write(cur_node);
                    } else {
                        if !previous.is_null() {
                            (*previous).next = (*head).next;
                        }
                        Global::default().deallocate(
                            NonNull::new(head as *mut u8).unwrap_unchecked(),
                            Layout::new::<Node>(),
                        );
                    }
                    return Ok(old_start as *mut u8);
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
                let head_begin = val.start;
                let head_end = val.start + val.len as u64;
                if head_begin == end {
                    val.start = begin;
                    val.len += len;
                    head.write(val);
                    return Ok(());
                } else if head_end == begin {
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

pub struct FreeListAllocator(spin::Mutex<FreeList>);

impl FreeListAllocator {
    pub fn push_region(&self, adr: u64, len: usize) -> Result<()> {
        self.0.lock().push_region(adr, len)
    }
}

unsafe impl Allocator for FreeListAllocator {
    fn allocate(&self, layout: Layout) -> core::result::Result<NonNull<[u8]>, AllocError> {
        match self
            .0
            .lock()
            .alloc_aligned_bytes(layout.align(), layout.size())
        {
            Ok(ptr) => Ok(NonNull::new(unsafe {
                slice::from_raw_parts_mut(ptr, layout.size()) as *mut [u8]
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
