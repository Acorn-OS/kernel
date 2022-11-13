use core::arch::global_asm;

global_asm!(include_str!("amd64.s"));

extern "C-unwind" {
    fn walk_stack(addresses: *mut u64, max_len: u64) -> u64;
}

#[derive(Debug)]
pub struct StackWalk([u64; 512], u64);

impl StackWalk {
    pub fn len(&self) -> usize {
        self.1 as usize
    }

    pub fn unwind(&self, mut f: impl FnMut(u64, usize)) {
        for i in 0..self.len() {
            f(self.0[i], i)
        }
    }
}

#[inline(never)]
pub unsafe fn stack_walk() -> StackWalk {
    let mut addresses = StackWalk([0; 512], 0);
    addresses.1 = walk_stack(addresses.0.as_mut_ptr(), addresses.0.len() as u64);
    addresses
}

#[no_mangle]
pub extern "C" fn _Unwind_Resume() {}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
