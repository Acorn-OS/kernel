use alloc::alloc::{AllocError, Allocator, Global};
use alloc::slice;
use alloc::string::ToString;
use core::alloc::{GlobalAlloc, Layout};
use core::fmt::{Debug, Display};
use core::ops::{Deref, DerefMut};
use core::ptr::{null_mut, NonNull};
use spin::Mutex;

#[derive(Debug)]
pub enum Error {
    ReserveError { adr: u64, len: usize },
    InsufficientSpace,
    AllocError(AllocError),
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&match self {
            Error::ReserveError { adr, len } => {
                format!("reserve error: (adr: 0x{adr:016x}, len: 0x{len:x})",)
            }
            Error::InsufficientSpace => "insufficient space".to_string(),
            Error::AllocError(e) => format!("alloc error: {e:?}"),
        })
    }
}

impl core::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;

impl From<AllocError> for Error {
    fn from(value: AllocError) -> Self {
        Self::AllocError(value)
    }
}

#[derive(Debug)]
pub struct Node {
    start: u64,
    len: usize,
    next: *mut Self,
}

impl Node {
    pub fn new(start: u64, len: usize) -> Self {
        debug_assert!(
            start as u128 + len as u128 <= u64::MAX as u128 + 1,
            "invalid range (start: 0x{start:016x}, len: 0x{len:x})"
        );
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
    #[cfg(feature = "log")]
    log::debug!(
        "freelist node(0x{:16x}) alloc: (start: {:016x}, size: {:x})",
        alloc as u64,
        node.start,
        node.len
    );
    unsafe { alloc.write(node) };
    Ok(alloc)
}

fn node_free<A: Allocator>(node: *mut Node, a: &A) {
    let layout = Layout::new::<Node>();
    unsafe { a.deallocate(NonNull::new_unchecked(node as *mut u8), layout) }
    #[cfg(feature = "log")]
    log::debug!("freelist node(0x{:16x}) free", node as u64,);
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
                            (*head).start + ((*head).len as u64 - 1),
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

    unsafe fn remove_node(&mut self, previous: *mut Node, node: *mut Node) {
        if !previous.is_null() {
            (*previous).next = (*node).next;
        } else {
            self.head = (*node).next;
        }
        node_free(node, &self.allocator);
    }

    fn contains_range(&self, start: u64, len: usize) -> bool {
        let mut head = self.head;
        let end = start as u128 + len as u128;
        while !head.is_null() {
            let cur = unsafe { head.read() };
            let cur_start = cur.start;
            let cur_end = cur_start as u128 + cur.len as u128;
            if (start >= cur_start && (start as u128) < cur_end)
                || (end > cur_start as u128 && end <= cur_end)
            {
                return true;
            }
            head = unsafe { (*head).next };
        }
        false
    }

    pub fn push_region(&mut self, adr: u64, len: usize) -> Result<()> {
        let node = node_alloc(Node::new(adr, len), &self.allocator)?;
        unsafe { self.insert_node(node) };
        Ok(())
    }

    pub unsafe fn push_node_raw(&mut self, node: *mut Node) {
        debug_assert!(!node.is_null(), "attempted to allocate invalid node");
        debug_assert!((*node).next.is_null());
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
                let aligned_end = aligned_start + len as u64;
                let left_start = cur_node.start;
                let left_end = aligned_start;
                let left_len = (left_end - left_start) as usize;
                let right_start = aligned_end;
                let right_end = cur_node.start + cur_node.len as u64;
                let right_len = (right_end.saturating_sub(right_start)) as usize;
                if right_end >= aligned_end {
                    if aligned_start == left_start {
                        cur_node.len -= len;
                        cur_node.start += len as u64;
                        if cur_node.len > 0 {
                            head.write(cur_node);
                        } else {
                            self.remove_node(previous, head);
                        }
                    } else if left_len > 0 {
                        self.remove_node(previous, head);
                        self.push_region(left_start, left_len)?;
                        if right_len > 0 {
                            self.push_region(right_start, right_len)?;
                        }
                    }

                    #[cfg(feature = "log")]
                    log::debug!(
                        "freelist(0x{:016x}) alloc: {:016x}",
                        self as *const _ as u64,
                        aligned_start
                    );
                    return Ok(aligned_start as *mut u8);
                } else {
                    previous = head;
                    head = cur_node.next;
                }
            }
        }
        Err(Error::InsufficientSpace)
    }

    pub fn alloc_layout(&mut self, layout: Layout) -> Result<*mut u8> {
        self.alloc_aligned_bytes(layout.align(), layout.size())
    }

    pub fn alloc<T>(&mut self) -> Result<*mut T> {
        let layout = Layout::new::<T>();
        self.alloc_layout(layout).map(|v| v as *mut T)
    }

    unsafe fn insert_node(&mut self, node: *mut Node) {
        let mut cur = self.head;
        let mut previous = null_mut() as *mut Node;
        let begin = (*node).start;
        let len = (*node).len;
        if len <= 0 {
            return;
        }
        let end = begin as u128 + len as u128;
        while !cur.is_null() {
            let cur_begin = (*cur).start;
            let cur_end = (*cur).start as u128 + (*cur).len as u128;
            if cur_begin as u128 == end {
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
            } else if cur_end == begin as u128 {
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

    /// # Safety
    /// Unless `ptr` and `len` belongs to a previously allocated
    /// area of memory, it may cause undesired behaviour with the
    /// allocator allocating memory that doesn't belong to it.
    pub unsafe fn free_bytes(&mut self, ptr: *mut u8, len: usize) -> Result<()> {
        debug_assert!(
            !self.contains_range(ptr as u64, len),
            "attempting to double free region (start: 0x{:016x}, len: {len:x})",
            ptr as u64
        );
        let node = node_alloc(Node::new(ptr as u64, len), &self.allocator)?;
        unsafe {
            self.insert_node(node);
        }
        #[cfg(feature = "log")]
        log::debug!(
            "freelist(0x{:016x}) free: {:016x}",
            self as *const _ as u64,
            ptr as u64
        );
        Ok(())
    }

    /// # Safety
    /// Read `free_bytes`.
    pub unsafe fn free<T>(&mut self, ptr: *mut T) -> Result<()> {
        let layout = Layout::new::<T>();
        debug_assert!(ptr.is_aligned_to(layout.align()));
        self.free_bytes(ptr as *mut u8, layout.size())
    }

    pub fn reserve_bytes(&mut self, adr: u64, count: usize) -> Result<()> {
        let adr_beg = adr;
        if count <= 0 {
            return Ok(());
        }
        debug_assert!((adr_beg as u128 + count as u128 - 1) <= u64::MAX as u128);
        let adr_end = adr_beg as u128 + count as u128;
        let mut head = self.head;
        let mut prev = null_mut() as *mut Node;
        unsafe {
            while !head.is_null() {
                let cur_node = head.read();
                let left_start = cur_node.start;
                let left_end = adr_beg;
                let right_start = adr_end;
                let right_end = cur_node.start as u128 + cur_node.len as u128;
                if left_start <= left_end && right_start <= right_end {
                    let left_len = left_end.saturating_sub(left_start) as usize;
                    let right_len = right_end.saturating_sub(right_start) as usize;
                    self.remove_node(prev, head);
                    if left_len > 0 {
                        self.push_region(left_start, left_len)?;
                    }
                    if right_len > 0 {
                        self.push_region(right_start as u64, right_len)?;
                    }
                    return Ok(());
                }
                prev = head;
                head = (*head).next;
            }
        }
        Err(Error::ReserveError { adr, len: count })
    }
}

#[repr(transparent)]
pub struct FreeListAllocator<A: Allocator = Global>(Mutex<FreeList<A>>);

impl<A: Allocator> Deref for FreeListAllocator<A> {
    type Target = Mutex<FreeList<A>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<A: Allocator> DerefMut for FreeListAllocator<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

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
                Error::ReserveError { .. } => AllocError,
                Error::AllocError(err) => err,
            }),
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        assert!(ptr.as_ptr().is_aligned_to(layout.align()));
        match self.0.lock().free_bytes(ptr.as_ptr(), layout.size()) {
            Ok(()) => {}
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
        assert!(!ptr.is_null(), "attempting to deallocate a nullptr");
        self.deallocate(NonNull::new_unchecked(ptr), layout);
    }
}
