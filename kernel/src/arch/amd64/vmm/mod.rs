mod page_map;

use crate::boot::BOOT_INFO;
use core::mem::size_of;
use page_map::PageMap;

static mut PAGE_MAP: PageMap = PageMap::new();

const RESV_DEPTH: usize = 2;
const RESV_SIZE: usize = (RESV_DEPTH + 1) * 512 * 8;
static mut RESV: [u8; RESV_SIZE] = [0; RESV_SIZE];

#[ctor]
unsafe fn init() {
    info!("configuring mapping");
    let mut resv_i = 0;
    let mut resv = || {
        if resv_i > RESV_SIZE - size_of::<PageMap>() {
            panic!("out of reserved page size; allocated [{resv_i}] out of [{RESV_SIZE}]");
        }
        let ptr = &mut RESV[resv_i] as *mut _ as *mut PageMap;
        resv_i += size_of::<PageMap>();
        ptr
    };
    // Identity map physical memory.
    info!("identity map the first ({RESV_DEPTH} * 512)GiB  of physical memory");
    let mut phys_adr = 0;
    for i0 in 0..RESV_DEPTH {
        for i1 in 0..512 {
            PAGE_MAP.alloc_map2_virt((i0, i1), &mut resv, {
                let ret = phys_adr;
                phys_adr += 1 << 30;
                ret
            })
        }
    }
    // Map kernel to high address.
    info!("mapping kernel to higher address");
    let mut phys_adr = BOOT_INFO.kernel_phys_base;
    for i1 in 0..512 {
        PAGE_MAP.alloc_map2_virt((511, i1), &mut resv, {
            let ret = phys_adr;
            phys_adr += 1 << 30;
            ret
        })
    }
    info!("installing page map");
    info!(
        "map: {:016X}",
        (&PAGE_MAP as *const _ as u64) - BOOT_INFO.kernel_virt_base + BOOT_INFO.kernel_phys_base
    );
    loop {}
    // TODO! map 1111 1111 1111 [1 1111 1111] [1 1111 1111] [1 1100 0000] [0 0000 0000] [0000 0000 0000] using 4KiB pagemaps!
    let mut ptr = &PAGE_MAP as *const _ as u64;
    ptr -= BOOT_INFO.kernel_virt_base;
    ptr += BOOT_INFO.kernel_phys_base;
    PageMap::install_ptr(ptr as *const _);
}
