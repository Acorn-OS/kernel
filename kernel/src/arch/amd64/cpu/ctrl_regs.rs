pub mod cr0 {
    use core::arch::asm;

    pub const PE: u64 = 1 << 0;
    pub const MP: u64 = 1 << 1;
    pub const EM: u64 = 1 << 2;
    pub const TS: u64 = 1 << 3;
    pub const ET: u64 = 1 << 4;
    pub const NE: u64 = 1 << 5;
    pub const WP: u64 = 1 << 16;
    pub const AM: u64 = 1 << 18;
    pub const NW: u64 = 1 << 29;
    pub const CD: u64 = 1 << 30;
    pub const PG: u64 = 1 << 31;

    pub fn get() -> u64 {
        unsafe {
            let out: u64;
            asm!("mov rax, cr0", out("rax") out);
            out
        }
    }
}

pub mod cr4 {
    use core::arch::asm;

    pub fn get() -> u64 {
        unsafe {
            let out: u64;
            asm!("mov rax, cr4", out("rax") out);
            out
        }
    }
}
