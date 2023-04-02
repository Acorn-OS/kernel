use super::vm::PageMap;
use core::arch::asm;

#[repr(C)]
pub struct Core {}

static mut CORE: Core = Core {};

pub fn get_core() -> *mut Core {
    todo!()
}

pub fn cur_pgtbl() -> *mut PageMap {
    let ptr;
    unsafe {
        asm!(
        "mov rax, cr3",
        out("rax") ptr, 
        options(nostack));
    }
    ptr
}
