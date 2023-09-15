use core::cell::UnsafeCell;
use core::fmt::{self, Debug};
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

use crate::arch::interrupt;

#[derive(Debug)]
pub struct LockPrimitive {
    inner: AtomicBool,
    empty_lock: UnsafeCell<()>,
    enable_irq: AtomicBool,
}

unsafe impl Send for LockPrimitive {}
unsafe impl Sync for LockPrimitive {}

impl LockPrimitive {
    pub const fn new() -> Self {
        Self {
            inner: AtomicBool::new(false),
            empty_lock: UnsafeCell::new(()),
            enable_irq: AtomicBool::new(false),
        }
    }

    pub fn is_locked(&self) -> bool {
        self.inner.load(Ordering::SeqCst)
    }

    pub unsafe fn manually_lock(&self) {
        let irq_enable = interrupt::is_enabled();
        interrupt::disable();
        while self
            .inner
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {}
        self.enable_irq.store(irq_enable, Ordering::Relaxed);
    }

    pub fn manually_unlock(&self) {
        if self
            .inner
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
            && self
                .enable_irq
                .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
        {
            interrupt::enable();
        }
    }

    pub fn lock_empty<'a>(&'a self) -> LockGuard<()> {
        self.lock(&self.empty_lock)
    }

    pub fn lock<'a, T>(&'a self, data: &'a UnsafeCell<T>) -> LockGuard<T> {
        LockGuard {
            inner: data,
            lock: self,
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
        })
    }
}

pub struct LockGuard<'a, T> {
    inner: &'a UnsafeCell<T>,
    lock: &'a LockPrimitive,
}

impl<'a, T: Debug> Debug for LockGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LockGuard")
            .field("inner", unsafe { &*self.inner.get() })
            .field("lock", &self.lock)
            .finish()
    }
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
        self.lock.manually_unlock()
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct ManualLock {
    inner: LockPrimitive,
}

unsafe impl Send for ManualLock {}
unsafe impl Sync for ManualLock {}

impl ManualLock {
    pub const fn new() -> Self {
        Self {
            inner: LockPrimitive::new(),
        }
    }

    /// # Safety
    /// if the thread is never unlocked and is then dropped, some threads
    /// may spinloop forever.
    pub unsafe fn lock(&self) {
        self.inner.manually_lock()
    }

    pub fn unlock(&self) {
        self.inner.manually_unlock()
    }

    pub fn do_locked<R>(&self, mut f: impl FnMut() -> R) -> R {
        unsafe { self.lock() }
        let r = f();
        self.unlock();
        r
    }
}

#[derive(Debug)]
pub struct Locked<T> {
    lock: LockPrimitive,
    inner: UnsafeCell<T>,
}

unsafe impl<T> Sync for Locked<T> {}
unsafe impl<T> Send for Locked<T> {}

impl<T> Locked<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
            lock: LockPrimitive::new(),
        }
    }

    pub fn lock<'a>(&'a self) -> LockGuard<'a, T> {
        self.lock.lock(&self.inner)
    }
}

#[derive(Debug)]
pub struct ThreadLocked<T> {
    lock: LockPrimitive,
    inner: UnsafeCell<T>,
}

unsafe impl<T> Sync for ThreadLocked<T> {}
unsafe impl<T> Send for ThreadLocked<T> {}

impl<T> ThreadLocked<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
            lock: LockPrimitive::new(),
        }
    }

    pub fn lock<'a>(&'a self) -> LockGuard<'a, T> {
        // TODO: allow the same thread to lock multiple times.
        // do not lock anything rn, since we just got one thread.
        self.lock.lock(&self.inner)
    }
}
