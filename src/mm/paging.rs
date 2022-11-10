pub const KERNEL_PG_SHIFT: usize = 12;
pub const KERNEL_PG_SIZE: usize = 1 << KERNEL_PG_SHIFT;
pub const KERNEL_PG_MASK: usize = KERNEL_PG_SIZE - 1;

use crate::{arch::paging, ksyms};

pub fn init() {
    // Remap kernel into virtual memory.
    paging::map(
        ksyms::virt_adr_start()..=ksyms::virt_adr_end(),
        0,
        paging::PageSize::Huge,
    );
    // Identity map kernel memory.
    paging::map(
        0..=ksyms::free_mem_adr() + ksyms::free_mem_len() - 1,
        0,
        paging::PageSize::Large,
    );
    unsafe { paging::install() };
}
