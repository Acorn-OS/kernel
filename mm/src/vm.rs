pub const KERNEL_PG_SHIFT: usize = 12;
pub const KERNEL_PG_SIZE: usize = 1 << KERNEL_PG_SHIFT;
pub const KERNEL_PG_MASK: usize = KERNEL_PG_SIZE - 1;

pub use libmm::vm::{kvma_end, kvma_start, kwm_end, kwm_start};

pub unsafe fn kvma_len() -> usize {
    kvma_end() - kvma_start() + 1
}

pub unsafe fn kwm_len() -> usize {
    kwm_end() - kwm_start() + 1
}
