#[repr(C)]
pub struct Core {}

static mut CORE: Core = Core {};

pub fn get_core() -> *mut Core {
    todo!()
}
