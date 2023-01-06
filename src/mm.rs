pub mod vm;
pub mod wm;

mod heap;
mod kalloc;

pub fn init() {
    unsafe {
        trace_init!("mm::wm");
        wm::init();
        trace_init!("mm::kalloc");
        kalloc::init();
        trace_init!("mm::vm");
        vm::init();
    }
}
