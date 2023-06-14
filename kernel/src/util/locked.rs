use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

use crate::arch::interrupt;

#[derive(Debug)]
#[repr(transparent)]
pub struct Lock {
    inner: AtomicBool,
}

impl Lock {
    pub const fn new() -> Self {
        Self {
            inner: AtomicBool::new(false),
        }
    }

    pub fn locked(&self) -> bool {
        self.inner.load(Ordering::SeqCst)
    }

    /// returns an error if already locked.
    pub unsafe fn manually_lock(&self) {
        self.inner.store(true, Ordering::SeqCst)
    }

    pub unsafe fn manually_unlock(&self) {
        self.inner.store(false, Ordering::SeqCst)
    }

    pub fn lock<'a, T>(&'a self, data: &'a UnsafeCell<T>) -> LockGuard<T> {
        let enable_irq = interrupt::is_enabled();
        interrupt::disable();
        while self
            .inner
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {}
        LockGuard {
            inner: data,
            lock: self,
            enable_irq,
        }
    }

    pub fn try_lock<'a, T>(&'a self, data: &'a UnsafeCell<T>) -> Option<LockGuard<T>> {
        let enable_irq = interrupt::is_enabled();
        interrupt::disable();
        if self
            .inner
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            if enable_irq {
                interrupt::enable()
            }
            return None;
        };
        Some(LockGuard {
            inner: data,
            lock: self,
            enable_irq,
        })
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct LockGuard<'a, T> {
    inner: &'a UnsafeCell<T>,
    lock: &'a Lock,
    enable_irq: bool,
}

impl<'a, T: 'a> Deref for LockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.get() }
    }
}

impl<'a, T: 'a> DerefMut for LockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner.get() }
    }
}

impl<'a, T> Drop for LockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.inner.store(false, Ordering::SeqCst);
        if self.enable_irq {
            interrupt::enable();
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Locked<T> {
    lock: Lock,
    inner: UnsafeCell<T>,
}

unsafe impl<T> Sync for Locked<T> {}
unsafe impl<T> Send for Locked<T> {}

impl<T> Locked<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
            lock: Lock::new(),
        }
    }

    pub fn lock<'a>(&'a self) -> LockGuard<'a, T> {
        self.lock.lock(&self.inner)
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct ThreadLocked<T> {
    lock: Lock,
    inner: UnsafeCell<T>,
}

unsafe impl<T> Sync for ThreadLocked<T> {}
unsafe impl<T> Send for ThreadLocked<T> {}

impl<T> ThreadLocked<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
            lock: Lock::new(),
        }
    }

    pub fn lock<'a>(&'a self) -> LockGuard<'a, T> {
        // TODO: allow the same thread to lock multiple times.
        // do not lock anything rn, since we just got one thread.
        self.lock.lock(&self.inner)
    }
}
