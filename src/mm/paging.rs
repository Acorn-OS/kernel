pub const KERNEL_PG_SHIFT: usize = 12;
pub const KERNEL_PG_SIZE: usize = 1 << KERNEL_PG_SHIFT;
pub const KERNEL_PG_MASK: usize = KERNEL_PG_SIZE - 1;
