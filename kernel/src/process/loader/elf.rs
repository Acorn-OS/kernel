use crate::mm::vmm::VirtualMemory;
use crate::mm::{heap, pmm};
use crate::process::thread::Thread;
use crate::process::{self, Process, ProcessId};
use core::ptr::NonNull;
use elf::Elf64;

pub fn spawn_kernel(elf: &Elf64) -> (NonNull<Process>, ProcessId) {
    let vmm = unsafe { map(elf, VirtualMemory::new_userland()) };
    debug!("entry address: {:x}", elf.program_entry());
    process::new_kernel_proc(heap::alloc(vmm), unsafe {
        Thread::new(pmm::alloc_pages(256).as_virt_ptr(), elf.program_entry())
    })
}

pub fn spawn_usrspc(elf: &Elf64) -> (NonNull<Process>, ProcessId) {
    let _vmm = unsafe { map(elf, VirtualMemory::new_userland()) };
    unimplemented!()
}

unsafe fn map(elf: &Elf64, mut vmm: VirtualMemory) -> VirtualMemory {
    // load program headers.
    for program_header in elf.program_headers() {
        let bytes = program_header.p_memsz;
        let vadr = program_header.p_vaddr;
        let off = program_header.p_offset;
        if vadr == 0 {
            continue;
        }
        debug!("loading elf program header: {program_header:x?}");
        let pages = bytes.div_ceil(pmm::PAGE_SIZE as u64) as usize;
        debug!("allocating pages '{pages}'");
        let alloced = pmm::alloc_pages(pages);
        let alloced_vptr = alloced.as_virt_ptr::<u8>();
        alloced_vptr.copy_from(elf.file_image().as_ptr().add(off as usize), bytes as usize);
        vmm.map_pages_raw(pages, vadr, alloced.adr());
    }
    vmm
}
