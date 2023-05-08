use crate::arch::vm;
use crate::mm::vmm::{Flags, VirtualMemory};
use crate::mm::{heap, pmm};
use crate::process::thread::Thread;
use crate::process::{self, Process, ProcessId};
use core::ptr::NonNull;
use elf::Elf64;

pub fn spawn(elf: &Elf64) -> (NonNull<Process>, ProcessId) {
    let mut vmm = unsafe { map(elf, VirtualMemory::new_userland()) };
    let thread = Thread::new(&mut vmm, elf.program_entry(), 256);
    process::new_kernel_proc(heap::alloc(vmm), thread)
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
        let pages = bytes.div_ceil(pmm::PAGE_SIZE as u64) as usize;
        let alloced = pmm::alloc_pages(pages);
        let alloced_vptr = alloced.as_virt_ptr::<u8>();
        alloced_vptr.copy_from(elf.as_ptr().add(off as usize), bytes as usize);
        vmm.map(
            Some(vadr),
            pages,
            Flags::Phys {
                flags: vm::Flags::PRESENT | vm::Flags::RW | vm::Flags::USER,
                phys: alloced.phys_adr(),
            },
        );
    }
    vmm
}
