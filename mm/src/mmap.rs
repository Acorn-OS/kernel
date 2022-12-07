#[repr(C)]
pub struct VirtMap {
    /// Force specific physical address.
    pub f_phy_adr: bool,
    /// Physical address used with `f_phy_adr`.
    pub phys_adr: usize,
}

#[repr(C)]
pub enum MapSize {
    /// Small page size used by a user process.
    UserSmall,
    /// Large page size used by a user process.
    UserLarge,
    /// Small page size used by the kernel.
    KernelSmall,
    /// Large page size used by the kernel.
    KernelLarge,
}

#[repr(C)]
pub enum MMapTy {
    /// Map to physical memory.
    Virtual(VirtMap),
}

#[repr(C)]
pub struct MMap {
    pub ty: MMapTy,
    pub size: MapSize,
}

#[repr(C)]
pub struct MUnmap {
    pub size: MapSize,
}

#[repr(C)]
pub enum MMapResult {
    Ok(u64),
}

#[repr(C)]
pub enum MUnmapResult {
    Ok(u64),
}

extern "C" {
    /// Returns pages mapped.
    pub fn mmap(adr: usize, len: usize, mmap: MMap) -> MMapResult;

    /// Returns pages unmapped.
    pub fn munmap(adr: usize, len: usize, munmap: MUnmap) -> MUnmapResult;
}
