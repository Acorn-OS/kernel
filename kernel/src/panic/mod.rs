#![panic_runtime]

#[cfg(debug_assertions)]
mod stack_unwind;

#[panic_handler]
pub unsafe fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    util::di();
    error!("{info}");
    #[cfg(debug_assertions)]
    {
        let stack_walk = stack_unwind::stack_walk();
        stack_walk.unwind(|adr, i| error!("[{i}]: 0x{adr:016X}"));
    }
    loop {
        util::di();
        util::halt()
    }
}
