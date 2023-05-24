use crate::arch::imp::cpu;
use core::arch::asm;

#[inline(always)]
pub fn print_regs() {
    unsafe {
        let cr4: u64;
        asm!("mov rax, cr4", out("rax") cr4);
        let cr3: u64;
        asm!("mov rax, cr3", out("rax") cr3);
        let cr0: u64;
        asm!("mov rax, cr0", out("rax") cr0);
        let gs_base = cpu::get_gs_base().adr();
        let gs_kernel_base = cpu::get_kernel_gs_base().adr();
        let fs: u16;
        asm!("mov ax, fs", out("ax") fs);
        let gs: u16;
        asm!("mov ax, gs", out("ax") gs);
        let cs: u16;
        asm!("mov ax, cs", out("ax") cs);
        let ds: u16;
        asm!("mov ax, ds", out("ax") ds);
        let ss: u16;
        asm!("mov ax, ss", out("ax") ss);
        info!("cr4:             0x{cr4:016x}");
        info!("cr3:             0x{cr3:016x}");
        info!("cr0:             0x{cr0:016x}");
        info!("gs_base:         0x{gs_base:016x}");
        info!("gs_kernel_base:  0x{gs_kernel_base:016x}");
        info!("fs:              0x{fs:04}");
        info!("gs:              0x{gs:04}");
        info!("cs:              0x{cs:04}");
        info!("ds:              0x{ds:04}");
        info!("ss:              0x{ss:04}");
    }
}
