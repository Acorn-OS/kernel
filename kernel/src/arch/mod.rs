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

    pub use vm::AllocSize;
    pub use vm::PageMap;
    pub use vm::PageMapEntry;

    pub const PAGE_SIZE: usize = vm::PAGE_SIZE;

    export_assert_fn!(vm::alloc_pages: unsafe fn(*mut PageMap, u64, usize, u64));
    export_assert_fn!(vm::alloc_large_pages: unsafe fn(*mut PageMap, u64, usize, u64));
    export_assert_fn!(vm::free_pages: unsafe fn(*mut PageMap, u64, usize));
    export_assert_fn!(vm::install: unsafe fn(*mut PageMap));
    export_assert_fn!(vm::new_page_map: fn() -> *mut PageMap);
    export_assert_fn!(
        vm::get_page_entry: unsafe fn(*mut PageMap, u64) -> Option<*mut PageMapEntry>
    );
    export_assert_fn!(vm::resv_pages: unsafe fn(*mut PageMap, u64, usize));
}

pub mod cpuc {
    use super::imp::cpuc;
    use crate::mm::vmm::VirtualMemory;

    pub use cpuc::Core;

    assert_fn!(Core::vmm: fn(&Core) -> *mut VirtualMemory);
    assert_fn!(Core::set_vmm: fn(&mut Core, *mut VirtualMemory));

    export_assert_fn!(cpuc::get: fn() -> *mut Core);
}

pub mod fb {
    use super::imp::fb;

    pub const WIDTH: usize = fb::WIDTH;
    pub const HEIGHT: usize = fb::HEIGHT;

    pub type Cursor = fb::Cursor;

    export_assert_fn!(fb::putb: fn(pos: Cursor, b: u8));
}
