use super::apic::lapic;
use super::apic::LApicPtr;
use super::gdt::Gdt;
use super::idt::Idt;
use super::msr;
use crate::mm::heap;
use core::arch::asm;
use core::ptr::null_mut;
use core::ptr::NonNull;

const KERNEL_GS_BASE: u32 = 0xC0000102;
const GS_BASE: u32 = 0xC0000101;

#[repr(C)]
pub struct Core {
    pub(super) lapic_ptr: LApicPtr,
    pub(super) idt_ptr: NonNull<Idt>,
    pub(super) gdt_ptr: NonNull<Gdt>,
}

pub unsafe fn swap() {
    asm!("swapgs");
}

pub unsafe fn set_kernel_gs_base(ptr: NonNull<Core>) {
    msr::set(KERNEL_GS_BASE, ptr.addr().get() as u64);
}

pub unsafe fn set_gs_base(ptr: *mut Core) {
    msr::set(GS_BASE, ptr as u64);
}

pub fn get() -> *mut Core {
    msr::get(KERNEL_GS_BASE) as *mut _
}

pub unsafe fn init_for_core() {
    trace!("initializing GDT");
    let mut gdt_ptr = heap::alloc(Gdt::new());
    gdt_ptr.as_mut().install();
    trace!("initializing IDT");
    let mut idt_ptr = heap::alloc(Idt::new());
    idt_ptr.as_mut().install();
    trace!("initializing LAPIC");
    let lapic_ptr = lapic::create_local();
    trace!("setting up cpuc object");
    set_kernel_gs_base(heap::alloc(Core {
        lapic_ptr,
        idt_ptr,
        gdt_ptr,
    }));
    set_gs_base(null_mut());
}
