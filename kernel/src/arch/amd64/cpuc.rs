use super::apic::lapic;
use super::apic::LApicPtr;
use super::gdt::{Gdt, TSS_SELECTOR};
use super::idt::Idt;
use super::msr;
use crate::arch::imp::gdt::Tss;
use crate::mm::heap;
use crate::mm::pmm;
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

pub fn get_kernel() -> Option<NonNull<Core>> {
    NonNull::new(msr::get(KERNEL_GS_BASE) as *mut _)
}

pub unsafe fn init_for_core() {
    trace!("initializing GDT");
    let rsp0_alloc = pmm::alloc_pages(16);
    let rsp1_alloc = pmm::alloc_pages(16);
    let rsp2_alloc = pmm::alloc_pages(16);
    let ist1_alloc = pmm::alloc_pages(16);
    let ist2_alloc = pmm::alloc_pages(16);
    let ist3_alloc = pmm::alloc_pages(16);
    let ist4_alloc = pmm::alloc_pages(16);
    let ist5_alloc = pmm::alloc_pages(16);
    let ist6_alloc = pmm::alloc_pages(16);
    let ist7_alloc = pmm::alloc_pages(16);
    let mut gdt_ptr = heap::alloc(Gdt::new(heap::alloc(Tss {
        rsp0: rsp0_alloc.virt_adr(),
        rsp1: rsp1_alloc.virt_adr(),
        rsp2: rsp2_alloc.virt_adr(),
        ist1: ist1_alloc.virt_adr(),
        ist2: ist2_alloc.virt_adr(),
        ist3: ist3_alloc.virt_adr(),
        ist4: ist4_alloc.virt_adr(),
        ist5: ist5_alloc.virt_adr(),
        ist6: ist6_alloc.virt_adr(),
        ist7: ist7_alloc.virt_adr(),
        iopb: 0,
        ..Tss::default()
    })));
    gdt_ptr.as_mut().install();
    gdt_ptr.as_mut().use_tss(TSS_SELECTOR);
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
