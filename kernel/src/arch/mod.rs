use crate::boot::BootInfo;

cfg_if! {
    if #[cfg(target_arch = "x86_64")]{
        mod amd64;
        use amd64 as imp;
    } else {
        static_assert!(false, "unsupported target")
    }
}

macro_rules! assert_fn {
    ($fn:path: $($tt:tt)*) => {
        const _: $($tt)* = $fn;
    };
}

macro_rules! assert_const {
    ($const:path : $ty:ty) => {
        const _: $ty = $const;
    };
}

macro_rules! export_assert_fn {
    ($fn:path: $($tt:tt)*) => {
        pub use $fn;
        assert_fn!($fn: $($tt)*);
    };
}

pub mod serial {
    use super::imp;

    pub mod uart {
        use super::imp::serial::uart;

        export_assert_fn!(uart::putb: fn(u8));
        export_assert_fn!(uart::putc: fn(char));
        export_assert_fn!(uart::puts: fn(&str));
    }
}

pub mod vm {
    use super::imp::vm;
    use crate::util::adr::{PhysAdr, VirtAdr};
    use core::ptr::NonNull;

    pub use vm::PageMapEntry;

    assert_fn!(PageMapEntry::adr: fn(&PageMapEntry) -> PhysAdr);
    assert_fn!(PageMapEntry::set_adr: fn(&mut PageMapEntry, PhysAdr));
    assert_fn!(PageMapEntry::present: fn(&PageMapEntry) -> bool);

    pub use vm::PageMapPtr;

    pub use vm::Flags;

    assert_const!(Flags::PRESENT: Flags);
    assert_const!(Flags::RW: Flags);
    assert_const!(Flags::USER: Flags);
    assert_const!(Flags::SIZE_MEDIUM: Flags);
    assert_const!(Flags::SIZE_LARGE: Flags);

    pub const PAGE_SIZE: usize = vm::PAGE_SIZE;
    pub const MEDIUM_PAGE_SIZE: usize = vm::MEDIUM_PAGE_SIZE;
    pub const LARGE_PAGE_SIZE: usize = vm::LARGE_PAGE_SIZE;

    export_assert_fn!(vm::map: unsafe fn(PageMapPtr, VirtAdr, usize, PhysAdr, Flags));
    export_assert_fn!(vm::unmap: unsafe fn(PageMapPtr, VirtAdr, usize));
    export_assert_fn!(vm::install: unsafe fn(PageMapPtr));
    export_assert_fn!(vm::new_userland_page_map: unsafe fn() -> PageMapPtr);
    export_assert_fn!(vm::kernel_page_map: unsafe fn() -> PageMapPtr);
    export_assert_fn!(
        vm::get_page_entry: unsafe fn(PageMapPtr, VirtAdr) -> Option<NonNull<PageMapEntry>>
    );
}

pub mod fb {
    use super::imp::fb;

    pub const WIDTH: usize = fb::WIDTH;
    pub const HEIGHT: usize = fb::HEIGHT;

    pub use fb::Cursor;

    export_assert_fn!(fb::putb: fn(pos: Cursor, b: u8));
}

pub mod interrupt {
    use super::imp::interrupt;
    use super::imp::vm::PageMapPtr;

    pub use interrupt::StackFrame;

    assert_fn!(StackFrame::new_kernel: fn(u64, u64, PageMapPtr) -> StackFrame);
    assert_fn!(StackFrame::new_userspace: fn(u64, u64, PageMapPtr) -> StackFrame);

    export_assert_fn!(interrupt::halt: fn());
    export_assert_fn!(interrupt::enable: fn());
    export_assert_fn!(interrupt::disable: fn());
}

pub mod stack_unwind {
    use super::imp::stack_unwind;

    pub use stack_unwind::StackFrame;

    assert_fn!(StackFrame::ip: fn(&StackFrame) -> u64);
    assert_fn!(StackFrame::next: fn(&StackFrame) -> *const StackFrame);
    assert_fn!(StackFrame::from_current_stackframe: unsafe fn() -> *const StackFrame);
}

pub mod thread {
    use super::imp::thread;
    use super::interrupt::StackFrame;
    use crate::process::Process;
    use crate::util::adr::VirtAdr;
    use core::ptr::NonNull;

    pub use thread::ThreadId;

    assert_fn!(ThreadId::new: fn(u64) -> ThreadId);

    pub use thread::Thread;

    assert_fn!(Thread::update_stackframe: unsafe fn(&mut Thread, StackFrame));
    assert_fn!(Thread::get_stackframe: fn(&Thread) -> StackFrame);

    export_assert_fn!(
        thread::new: unsafe fn(NonNull<Process>, ThreadId, VirtAdr, VirtAdr) -> NonNull<Thread>
    );

    export_assert_fn!(thread::free: unsafe fn(*mut Thread));
    export_assert_fn!(thread::cur_thread: fn() -> NonNull<Thread>);
    export_assert_fn!(thread::set_thread: unsafe fn(NonNull<Thread>));
}

pub mod panic {
    use super::imp::panic;

    export_assert_fn!(panic::print_regs: fn());
}

pub use imp::padr;
pub use imp::vadr;

export_assert_fn!(imp::arch_init: unsafe fn(&mut BootInfo));
