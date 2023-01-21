use core::alloc::Layout;

use crate::arch;

static mut CUR: usize = usize::MAX;

extern "C" {
    static __kwm: u8;
}

pub unsafe fn init() {
    /* TODO: remove constant */
    unsafe { CUR = &__kwm as *const _ as usize };
}

pub unsafe fn reserve_amount(amount: usize) -> *mut u8 {
    if CUR.saturating_add(amount) < arch::mm::adr::KVIRT_END {
        let ptr = CUR as *mut u8;
        CUR += amount;
        ptr
    } else {
        panic!("Out of allocatable work memory for the kernel. {amount}")
    }
}

unsafe fn align(align: usize) {
    let pad = CUR as usize % align;
    let pad = align - pad;
    CUR += pad;
}

pub unsafe fn reserve<T>(f: impl Fn() -> T) -> *const T {
    reserve_mut(f)
}

pub unsafe fn reserve_mut<T>(f: impl Fn() -> T) -> *mut T {
    let layout = Layout::new::<T>();
    align(layout.align());
    let ptr = reserve_amount(layout.size()) as *mut T;
    *ptr = f();
    ptr
}
