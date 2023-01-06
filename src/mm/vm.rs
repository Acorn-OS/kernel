use crate::arch::mm::{self, MapFlags, PageMapPtr};

const VIRT_LO: usize = 0xFFFFFFFF80000000;

static mut KMAP: PageMapPtr = 0;

pub fn init() {
    let kmap = mm::new_map();
    // Identity map the first 512GiB.
    debug!("Identity mapping the first 512GiB.");
    mm::map_range(kmap, 0, 512 << 30, 0, MapFlags { huge: true }).expect("mmap failed");
    // Map the upper 2GiB into virtual memory.
    debug!("Mapping the kernel to the uppper 2GiB of virtual memory.");
    mm::map_range(
        kmap,
        VIRT_LO,
        2 << 30, // 2GiB
        0,
        MapFlags { huge: true },
    )
    .expect("mmap failed");
    // Install the new page table.
    debug!("Installing new page table.");
    mm::swap_map(kmap);
    unsafe { KMAP = kmap };
}

/// Virtual to physical (kernel only).
#[inline]
pub fn kv2p(virt: usize) -> usize {
    debug_assert!(virt >= VIRT_LO);
    if virt < VIRT_LO {
        0
    } else {
        virt - VIRT_LO
    }
}
