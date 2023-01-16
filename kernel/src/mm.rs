pub mod wm;

mod heap;
mod kalloc;

pub fn init() {
    unsafe {
        wm::init();
        kalloc::init();
    }
}
